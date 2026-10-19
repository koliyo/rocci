// The SQL profile is deliberately narrower than an operating-system SQLite.
// Parse parameter tokens; do not rewrite SQL expressions or interpolate values.
export function statementInfo(query) {
  if (
    typeof query !== "string" ||
    new TextEncoder().encode(query).length > 1024 * 1024
  )
    throw new Error("SQL statement exceeds 1 MiB");
  const names = [],
    parameters = [];
  let word = "",
    first = "",
    ended = false,
    end = query.length;
  for (let i = 0; i < query.length; ) {
    const before = i;
    const c = query[i];
    if (/\s/.test(c)) i++;
    else if (c === "-" && query[i + 1] === "-") {
      i += 2;
      while (i < query.length && query[i] !== "\n") i++;
    } else if (c === "/" && query[i + 1] === "*") {
      i += 2;
      while (i < query.length && !(query[i] === "*" && query[i + 1] === "/"))
        i++;
      if (i === query.length) throw new Error("Unclosed SQL comment");
      i += 2;
    } else {
      if (ended) throw new Error("Only one SQL statement is supported");
      if (c === ";") {
        ended = true;
        end = i;
        i++;
      } else if (c === "'" || c === '"' || c === "`" || c === "[") {
        const close = c === "[" ? "]" : c;
        i++;
        let closed = false;
        while (i < query.length) {
          if (query[i++] === close) {
            if (query[i] === close && close !== "]") {
              i++;
              continue;
            }
            closed = true;
            break;
          }
        }
        if (!closed) throw new Error("Unclosed SQL string or identifier");
      } else if (c === ":") {
        const start = i++;
        while (i < query.length && /[A-Za-z0-9_]/.test(query[i])) i++;
        if (i === start + 1) throw new Error("SQL parameter must have a name");
        const name = query.slice(start, i);
        if (!names.includes(name)) names.push(name);
        parameters.push({ start, end: i, name });
      } else if (c === "?" || c === "$" || c === "@")
        throw new Error("Worker SQL supports named :field parameters only");
      else if (/[A-Za-z_]/.test(c)) {
        const start = i++;
        while (i < query.length && /[A-Za-z0-9_]/.test(query[i])) i++;
        word = query.slice(start, i).toUpperCase();
        if (!first) first = word;
        if (
          [
            "ATTACH",
            "DETACH",
            "BEGIN",
            "COMMIT",
            "ROLLBACK",
            "SAVEPOINT",
            "RELEASE",
            "RETURNING",
          ].includes(word)
        )
          throw new Error(`Unsupported Worker SQL operation: ${word}`);
      } else i++;
    }
    if (i <= before) throw new Error("SQL scanner did not advance");
  }
  if (
    ![
      "SELECT",
      "INSERT",
      "UPDATE",
      "DELETE",
      "CREATE",
      "DROP",
      "ALTER",
    ].includes(first)
  )
    throw new Error(`Unsupported Worker SQL statement: ${first}`);
  // Only a trailing terminator is removed, after token validation above.
  return {
    query: query.slice(0, end),
    names,
    parameters,
    read: first === "SELECT",
  };
}

function decodeBinding(value) {
  switch (value[0]) {
    case "null":
      return null;
    case "integer": {
      const integer = BigInt(value[1]);
      if (integer < -(1n << 63n) || integer >= 1n << 63n)
        throw new Error("Worker SQL integer parameter outside I64 range");
      return integer.toString();
    }
    case "real":
      if (!Number.isFinite(value[1])) throw new Error("Non-finite SQL number");
      return value[1];
    case "text":
      if (typeof value[1] !== "string") throw new Error("Invalid SQL text");
      return value[1];
    case "blob":
      return new Uint8Array(value[1]).buffer;
    default:
      throw new Error("Unknown SQL parameter type");
  }
}
function bind(statement, bindings) {
  const types = {
    integer: "INTEGER",
    real: "REAL",
    text: "TEXT",
    blob: "BLOB",
  };
  let query = "",
    cursor = 0;
  for (const token of statement.parameters) {
    const value = bindings.get(token.name);
    const placeholder = `?${statement.names.indexOf(token.name) + 1}`;
    const type = types[value.type];
    query +=
      statement.query.slice(cursor, token.start) +
      (type ? `CAST(${placeholder} AS ${type})` : placeholder);
    cursor = token.end;
  }
  query += statement.query.slice(cursor);
  return {
    query,
    values: statement.names.map((name) => bindings.get(name).value),
  };
}
export class ObjectSql {
  constructor(sql, aliases) {
    this.sql = sql;
    this.aliases = new Set(aliases);
  }
  open(path, options) {
    if (!this.sql)
      throw new Error("SQLite requires the durable-object execution profile");
    if (!this.aliases.has(path))
      throw new Error(`Unmapped logical SQLite path: ${path}`);
    // Native defaults map to one provider-managed database. Other native pool,
    // timeout, journal and sync controls have no equivalent in object storage.
    if (JSON.stringify(options) !== "[8,100,1000,32,1,0]")
      throw new Error(
        "Worker SQL accepts Sqlite.default_config only; native pool/journal options have no provider equivalent",
      );
    return { kind: "db" };
  }
  prepare(query) {
    return { kind: "statement", ...statementInfo(query), columns: [] };
  }
  execute(statement, params) {
    const bindings = new Map();
    for (const { name, value } of params) {
      if (bindings.has(name))
        throw new Error(`Duplicate SQL parameter: ${name}`);
      bindings.set(name, { type: value[0], value: decodeBinding(value) });
    }
    if (
      bindings.size !== statement.names.length ||
      statement.names.some((name) => !bindings.has(name))
    )
      throw new Error("Missing or extra named SQL parameters");
    const { query, values } = bind(statement, bindings);
    if (!statement.read) {
      const cursor = this.sql.exec(query, ...values);
      if (cursor.columnNames.length)
        throw new Error("SQL writes returning rows are unsupported");
      cursor.toArray();
      return { kind: "execution", rows: [], index: 0 };
    }
    // LIMIT 0 prepares column metadata without running the query. MATERIALIZED
    // evaluates each expression once before projecting its exact storage class
    // and integer text. Never decode an integer from a potentially rounded JS number.
    const metadata = this.sql.exec(
      `SELECT * FROM (\n${statement.query}\n) LIMIT 0`,
      ...statement.names.map(() => null),
    );
    const columns = [...metadata.columnNames];
    metadata.toArray();
    if (columns.some((name) => /:\d+$/.test(name)))
      throw new Error(
        "Ambiguous or duplicate SQL column names; use unique explicit aliases without :number suffixes",
      );
    const quote = (name) => `q."${name.replaceAll('"', '""')}"`;
    const projection = columns
      .flatMap((name) => [
        `typeof(${quote(name)})`,
        `CAST(${quote(name)} AS TEXT)`,
        quote(name),
      ])
      .join(",");
    const columnList = columns
      .map((name) => `"${name.replaceAll('"', '""')}"`)
      .join(",");
    const cursor = this.sql.exec(
      `WITH q(${columnList}) AS MATERIALIZED (\n${query}\n) SELECT ${projection} FROM q`,
      ...values,
    );
    const rows = [];
    let total = 0;
    for (const row of cursor.raw()) {
      if (rows.length >= 10_000)
        throw new Error("Worker SQL materialization exceeds 10000 rows");
      const values = columns.map((_, index) => {
        const [type, exact, value] = row.slice(index * 3, index * 3 + 3);
        switch (type) {
          case "integer": {
            const integer = BigInt(exact);
            if (integer < -(1n << 63n) || integer >= 1n << 63n)
              throw new Error("SQL integer outside I64 range");
            return ["integer", integer.toString()];
          }
          case "real":
            if (!Number.isFinite(value))
              throw new Error("Non-finite SQL result");
            return ["real", value];
          case "text":
            return ["text", value];
          case "blob":
            return ["blob", [...new Uint8Array(value)]];
          case "null":
            return ["null"];
          default:
            throw new Error("Unknown SQLite storage class");
        }
      });
      const bytes = values.reduce(
        (sum, value) =>
          sum +
          (value[0] === "text"
            ? new TextEncoder().encode(value[1]).length
            : value[0] === "blob"
              ? value[1].length
              : 8),
        0,
      );
      total += bytes;
      if (total > 1024 * 1024)
        throw new Error("Worker SQL materialization exceeds 1 MiB");
      rows.push({ values, bytes });
    }
    statement.columns = columns;
    return { kind: "execution", rows, index: 0 };
  }
}
