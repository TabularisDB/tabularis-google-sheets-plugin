//! Schema metadata. For Google Sheets:
//! - one "database" per connection (the spreadsheet itself)
//! - no schemas, views, routines, foreign keys, or indexes
//! - tables = sheet tabs
//! - columns = first row of each tab, with types inferred from rows 2..N

use serde_json::{json, Value};

use crate::handlers::spreadsheet_id;
use crate::rpc::{error_response, ok_response};
use crate::sheets::{get_sheet_data, get_sheet_names, infer_type};

pub fn get_databases(id: Value, params: &Value) -> Value {
    match spreadsheet_id(&id, params) {
        Ok(sid) => ok_response(id, json!([sid])),
        Err(resp) => resp,
    }
}

pub fn get_schemas(id: Value, _params: &Value) -> Value {
    ok_response(id, json!([]))
}

pub fn get_tables(id: Value, params: &Value) -> Value {
    let sid = match spreadsheet_id(&id, params) {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    match get_sheet_names(&sid) {
        Ok(names) => {
            let tables: Vec<Value> = names
                .into_iter()
                .map(|n| json!({ "name": n, "schema": null, "comment": null }))
                .collect();
            ok_response(id, json!(tables))
        }
        Err(e) => error_response(id, -32000, &e.to_string()),
    }
}

pub fn get_columns(id: Value, params: &Value) -> Value {
    let table = params
        .get("table")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let sid = match spreadsheet_id(&id, params) {
        Ok(s) => s,
        Err(resp) => return resp,
    };

    match get_sheet_data(&sid, &table) {
        Ok((headers, rows)) => {
            let sample: Vec<Vec<Value>> = rows.iter().take(100).cloned().collect();
            ok_response(id, build_columns(&headers, &sample))
        }
        Err(e) => error_response(id, -32000, &e.to_string()),
    }
}

pub fn get_foreign_keys(id: Value, _params: &Value) -> Value {
    ok_response(id, json!([]))
}

pub fn get_indexes(id: Value, _params: &Value) -> Value {
    ok_response(id, json!([]))
}

pub fn get_views(id: Value, _params: &Value) -> Value {
    ok_response(id, json!([]))
}

pub fn get_view_definition(id: Value, _params: &Value) -> Value {
    ok_response(id, Value::String(String::new()))
}

pub fn get_view_columns(id: Value, _params: &Value) -> Value {
    ok_response(id, json!([]))
}

pub fn get_routines(id: Value, _params: &Value) -> Value {
    ok_response(id, json!([]))
}

pub fn get_routine_parameters(id: Value, _params: &Value) -> Value {
    ok_response(id, json!([]))
}

pub fn get_routine_definition(id: Value, _params: &Value) -> Value {
    ok_response(id, Value::String(String::new()))
}

pub fn get_schema_snapshot(id: Value, params: &Value) -> Value {
    let sid = match spreadsheet_id(&id, params) {
        Ok(s) => s,
        Err(resp) => return resp,
    };

    let names = match get_sheet_names(&sid) {
        Ok(n) => n,
        Err(e) => return error_response(id, -32000, &e.to_string()),
    };

    let mut result: Vec<Value> = Vec::new();
    for name in &names {
        if let Ok((headers, rows)) = get_sheet_data(&sid, name) {
            let sample: Vec<Vec<Value>> = rows.iter().take(100).cloned().collect();
            result.push(json!({
                "name": name,
                "columns": build_columns(&headers, &sample),
                "foreign_keys": []
            }));
        }
    }
    ok_response(id, json!(result))
}

pub fn get_all_columns_batch(id: Value, params: &Value) -> Value {
    let sid = match spreadsheet_id(&id, params) {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let tables = params
        .get("tables")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut result = json!({});
    for t in &tables {
        if let Some(name) = t.as_str() {
            if let Ok((headers, rows)) = get_sheet_data(&sid, name) {
                let sample: Vec<Vec<Value>> = rows.iter().take(100).cloned().collect();
                result[name] = build_columns(&headers, &sample);
            }
        }
    }
    ok_response(id, result)
}

pub fn get_all_foreign_keys_batch(id: Value, _params: &Value) -> Value {
    ok_response(id, json!({}))
}

/// Build the column list for a sheet: a synthetic `_row` primary key
/// plus one column per header, with the type inferred from up to 100
/// sample rows.
fn build_columns(headers: &[String], sample: &[Vec<Value>]) -> Value {
    let mut cols = vec![json!({
        "name": "_row",
        "data_type": "INTEGER",
        "is_nullable": false,
        "default_value": null,
        "is_pk": true,
        "is_auto_increment": true,
        "character_maximum_length": null
    })];

    for (i, header) in headers.iter().enumerate() {
        let col_vals: Vec<Value> = sample
            .iter()
            .map(|r| r.get(i).cloned().unwrap_or(Value::Null))
            .collect();
        cols.push(json!({
            "name": header,
            "data_type": infer_type(&col_vals),
            "is_nullable": true,
            "default_value": null,
            "is_pk": false,
            "is_auto_increment": false,
            "character_maximum_length": null
        }));
    }
    json!(cols)
}

/// Shared by the CRUD handlers — they need to know the header order to
/// build a row or locate a column.
pub fn fetch_headers(sid: &str, table: &str) -> anyhow::Result<Vec<String>> {
    get_sheet_data(sid, table).map(|(headers, _)| headers)
}
