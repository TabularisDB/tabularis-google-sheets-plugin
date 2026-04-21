//! DDL generation + mutation.
//!
//! Google Sheets has no real schema concept — only `get_create_table_sql`
//! is meaningful (it synthesises a CREATE TABLE from the inferred column
//! types for the ER diagram / SQL preview). Everything else is explicitly
//! unsupported rather than "not implemented", because users should know
//! these operations cannot exist on this data source.

use serde_json::{json, Value};

use crate::handlers::metadata::fetch_headers;
use crate::handlers::spreadsheet_id;
use crate::rpc::{error_response, ok_response};
use crate::sheets::{get_sheet_data, infer_type};

pub fn get_create_table_sql(id: Value, params: &Value) -> Value {
    let table = params
        .get("table")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let sid = match spreadsheet_id(&id, params) {
        Ok(s) => s,
        Err(resp) => return resp,
    };

    let (headers, rows) = match get_sheet_data(&sid, &table) {
        Ok(v) => v,
        Err(e) => return error_response(id, -32000, &e.to_string()),
    };

    if let Err(e) = fetch_headers(&sid, &table) {
        return error_response(id, -32000, &e.to_string());
    }

    let mut col_defs = vec!["  \"_row\" INTEGER PRIMARY KEY".to_string()];
    for (i, h) in headers.iter().enumerate() {
        let vals: Vec<Value> = rows
            .iter()
            .take(100)
            .map(|r| r.get(i).cloned().unwrap_or(Value::Null))
            .collect();
        col_defs.push(format!("  \"{h}\" {}", infer_type(&vals)));
    }

    let sql = format!("CREATE TABLE \"{table}\" (\n{}\n);", col_defs.join(",\n"));
    ok_response(id, json!(sql))
}

pub fn get_add_column_sql(id: Value, _params: &Value) -> Value {
    unsupported(id, "Google Sheets does not support ALTER TABLE ADD COLUMN.")
}

pub fn get_alter_column_sql(id: Value, _params: &Value) -> Value {
    unsupported(id, "Google Sheets does not support ALTER TABLE MODIFY COLUMN.")
}

pub fn get_create_index_sql(id: Value, _params: &Value) -> Value {
    unsupported(id, "Google Sheets does not support indexes.")
}

pub fn get_create_foreign_key_sql(id: Value, _params: &Value) -> Value {
    unsupported(id, "Google Sheets does not support foreign keys.")
}

pub fn drop_index(id: Value, _params: &Value) -> Value {
    unsupported(id, "Google Sheets does not support indexes.")
}

pub fn drop_foreign_key(id: Value, _params: &Value) -> Value {
    unsupported(id, "Google Sheets does not support foreign keys.")
}

fn unsupported(id: Value, msg: &str) -> Value {
    error_response(id, -32601, msg)
}
