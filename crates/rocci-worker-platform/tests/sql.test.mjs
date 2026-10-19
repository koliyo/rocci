import assert from "node:assert/strict";
import test from "node:test";
import { DatabaseSync } from "node:sqlite";
import { ObjectSql, statementInfo } from "../runtime/sql.mjs";

// Real SQLite. The separate workerd lane qualifies the provider API itself.
function database() {
  const database = new DatabaseSync(":memory:");
  return new ObjectSql(
    {
      exec(query, ...params) {
        const statement = database.prepare(query);
        statement.setReadBigInts(true);
        const columnNames = statement.columns().map((column) => column.name);
        let rows = [];
        // Node's SQLite API takes a named object; the provider binds by index.
        // The only bound parameter in these SQLite fixtures is :value.
        const args =
          params.length && query.includes(":value")
            ? [{ ":value": params[0] }]
            : params;
        if (columnNames.length) rows = statement.all(...args);
        else statement.run(...args);
        return {
          columnNames,
          toArray: () => rows,
          raw: () =>
            rows.map((row) => columnNames.map((column) => row[column])),
        };
      },
    },
    ["./app.db"],
  );
}
test("parameter scanner ignores quoted tokens and comments and always terminates", () => {
  assert.deepEqual(
    statementInfo("SELECT ':ignored', :value, :value /* :other */ -- :last\n")
      .names,
    [":value"],
  );
  for (const text of [
    "SELECT '",
    "SELECT /*",
    "SELECT 1; DELETE FROM t",
    "SELECT ?",
    "ATTACH DATABASE :name",
    "BEGIN",
  ])
    assert.throws(() => statementInfo(text));
});
test("typed projection preserves exact I64 results and storage classes", () => {
  const sql = database();
  const execution = sql.execute(
    sql.prepare(
      "SELECT 9223372036854775807 AS exact, 1.0 AS real, '🌍' AS text, x'00ff' AS blob, NULL AS nil",
    ),
    [],
  );
  assert.deepEqual(execution.rows[0].values, [
    ["integer", "9223372036854775807"],
    ["real", 1],
    ["text", "🌍"],
    ["blob", [0, 255]],
    ["null"],
  ]);
});
test("logical aliases and options fail before database access", () => {
  const sql = database();
  assert.throws(() => sql.open("./unmapped.db", []), /Unmapped/);
  assert.throws(() => sql.open("./app.db", []), /default_config/);
  assert.equal(sql.open("./app.db", [8, 100, 1000, 32, 1, 0]).kind, "db");
});
test("bindings reject missing and extra values and preserve exact numeric types", () => {
  const sql = database();
  const statement = sql.prepare("SELECT :value AS value");
  for (const params of [
    [],
    [{ name: ":wrong", value: ["integer", "1"] }],
    [{ name: ":value", value: ["integer", "9223372036854775808"] }],
  ])
    assert.throws(() => sql.execute(statement, params));
  assert.deepEqual(
    sql.execute(statement, [{ name: ":value", value: ["integer", "3"] }])
      .rows[0].values,
    [["integer", "3"]],
  );
  assert.deepEqual(
    sql.execute(statement, [
      { name: ":value", value: ["integer", "9223372036854775807"] },
    ]).rows[0].values,
    [["integer", "9223372036854775807"]],
  );
  assert.deepEqual(
    sql.execute(statement, [{ name: ":value", value: ["real", 3] }]).rows[0]
      .values,
    [["real", 3]],
  );
});
test("materialization bounds rows and duplicate column names", () => {
  const sql = database();
  assert.throws(
    () => sql.execute(sql.prepare("SELECT 1 AS x, 2 AS x"), []),
    /duplicate/,
  );
});
