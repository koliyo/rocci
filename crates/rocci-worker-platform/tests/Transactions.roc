app [Context, program] {
    pf: platform "platform/main.roc",
    http: "https://github.com/roc-lang/http/releases/download/1.0.0/6ZUwqYhCS8PU9Mo6MF7oV82ET2o7KYb57CLKDq4cq4sS.tar.zst",
}

import pf.Server
import pf.Sqlite
import pf.Path
import pf.Sleep
import http.Response

Context : { db : Sqlite.Db }
program = { init!, respond!, shutdown! }

init! = || {
    db = Sqlite.open!(Sqlite.default_config(Path.utf8("./transactions.db"))) ? |_| Exit(1)
    Sqlite.execute!({db,query:"CREATE TABLE IF NOT EXISTS writes (n INTEGER NOT NULL)",params:{}}) ? |_| Exit(1)
    Ok({config:Server.default_config,context:{db}})
}

respond! = |request, {db}| {
    path = match request.target() {
        Resource({raw_path,..}) => raw_path
        _ => "/"
    }
    match path {
        "/values" => {
            row : { exact : I64, minimum : I64, real : F64, text : Str, null_type : Str }
            row = Sqlite.query!({db,query:"SELECT :exact AS exact, :minimum AS minimum, :real AS real, :text AS text, typeof(NULL) AS null_type",params:{exact:9223372036854775807.I64,minimum:-9223372036854775808.I64,real:1.0.F64,text:"🌍"},limits:Sqlite.default_query_limits}) ? |error| ServerErr(Str.inspect(error))
            blob : Sqlite.Blob
            blob = Sqlite.query!({db,query:"SELECT x'00ff' AS blob",params:{},limits:Sqlite.default_query_limits}) ? |error| ServerErr(Str.inspect(error))
            Ok(Server.respond(Response.from_status(200).with_body(Str.to_utf8(Str.inspect((row,blob.to_bytes()))))))
        }
        "/snapshot" => {
            count : { n : I64 }
            count = Sqlite.query!({db,query:"SELECT COUNT(*) AS n FROM writes",params:{},limits:Sqlite.default_query_limits}) ? |error| ServerErr(Str.inspect(error))
            Ok(Server.respond(Response.from_status(200).with_body(Str.to_utf8(count.n.to_str()))))
        }
        "/drop" => {
            write_and_drop!(db) ? |error| ServerErr(Str.inspect(error))
            Ok(Server.respond(Response.from_status(204)))
        }
        _ => {
            tx = Sqlite.begin!(db,Immediate) ? |error| ServerErr(Str.inspect(error))
            tx.execute!({query:"INSERT INTO writes VALUES (:n)",params:{n:9223372036854775807.I64}}) ? |error| ServerErr(Str.inspect(error))
            tx.execute!({query:"INSERT INTO writes VALUES (:n)",params:{n:2.I64}}) ? |error| ServerErr(Str.inspect(error))
            match path {
                "/commit" => tx.commit!() ? |error| ServerErr(Str.inspect(error))
                "/rollback" => tx.rollback!() ? |error| ServerErr(Str.inspect(error))
                "/typed-error" => tx.execute!({query:"INSERT INTO nonexistent VALUES (1)",params:{}}) ? |error| ServerErr(Str.inspect(error))
                "/trap" => crash "transaction trap qualification"
                "/unsupported-effect" => Sleep.millis!(1)
                "/post-commit-error" => {
                    tx.commit!() ? |error| ServerErr(Str.inspect(error))
                    return Err(ServerErr("after commit"))
                }
                _ => return Err(ServerErr("unknown transaction fixture"))
            }
            Ok(Server.respond(Response.from_status(204)))
        }
    }
}

write_and_drop! = |db| {
    tx = Sqlite.begin!(db,Immediate)?
    tx.execute!({query:"INSERT INTO writes VALUES (3)",params:{}})?
    tx.execute!({query:"INSERT INTO writes VALUES (4)",params:{}})?
    Ok({})
}

shutdown! = |_,_| Ok({})
