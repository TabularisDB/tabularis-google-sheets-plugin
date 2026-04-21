//! Row-level CRUD. Each sheet's synthetic `_row` column holds the
//! 1-based sheet row number — it's the only primary key this driver
//! supports, because the Sheets API indexes rows by position.

use serde_json::{json, Value};

use crate::handlers::metadata::fetch_headers;
use crate::handlers::spreadsheet_id;
use crate::rpc::{error_response, ok_response};
use crate::sheets::{
    append_row, col_letter, delete_row, get_sheet_id, update_cell, value_to_string,
};

pub fn insert_record(id: Value, params: &Value) -> Value {
    let table = params
        .get("table")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let data = params
        .get("data")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let sid = match spreadsheet_id(&id, params) {
        Ok(s) => s,
        Err(resp) => return resp,
    };

    let headers = match fetch_headers(&sid, &table) {
        Ok(h) => h,
        Err(e) => return error_response(id, -32000, &e.to_string()),
    };

    let row: Vec<String> = headers
        .iter()
        .map(|h| {
            data.get(h)
                .map(value_to_string)
                .unwrap_or_default()
        })
        .collect();

    match append_row(&sid, &table, row) {
        Ok(()) => ok_response(id, Value::Null),
        Err(e) => error_response(id, -32000, &e.to_string()),
    }
}

pub fn update_record(id: Value, params: &Value) -> Value {
    let table = params
        .get("table")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let pk_col = params
        .get("pk_col")
        .and_then(Value::as_str)
        .unwrap_or("_row");
    let pk_val_owned = params.get("pk_val").map(value_to_string).unwrap_or_default();
    let column = params
        .get("col_name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let value = params.get("new_val").map(value_to_string).unwrap_or_default();

    if pk_col != "_row" {
        return error_response(
            id,
            -32602,
            &format!("This driver only supports '_row' as primary key (got '{pk_col}')."),
        );
    }

    let row_num: usize = match pk_val_owned.parse() {
        Ok(n) => n,
        Err(_) => {
            return error_response(
                id,
                -32602,
                &format!("Invalid primary key value: '{pk_val_owned}'."),
            )
        }
    };

    let sid = match spreadsheet_id(&id, params) {
        Ok(s) => s,
        Err(resp) => return resp,
    };

    let headers = match fetch_headers(&sid, &table) {
        Ok(h) => h,
        Err(e) => return error_response(id, -32000, &e.to_string()),
    };

    let pos = match headers.iter().position(|h| h == &column) {
        Some(p) => p,
        None => {
            return error_response(
                id,
                -32602,
                &format!("Column '{column}' not found in sheet '{table}'."),
            )
        }
    };

    match update_cell(&sid, &table, &col_letter(pos + 1), row_num, &value) {
        Ok(()) => ok_response(id, json!(1)),
        Err(e) => error_response(id, -32000, &e.to_string()),
    }
}

pub fn delete_record(id: Value, params: &Value) -> Value {
    let table = params
        .get("table")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let pk_col = params
        .get("pk_col")
        .and_then(Value::as_str)
        .unwrap_or("_row");
    let pk_val_owned = params.get("pk_val").map(value_to_string).unwrap_or_default();

    if pk_col != "_row" {
        return error_response(
            id,
            -32602,
            &format!("This driver only supports '_row' as primary key (got '{pk_col}')."),
        );
    }

    let row_num: usize = match pk_val_owned.parse() {
        Ok(n) => n,
        Err(_) => {
            return error_response(
                id,
                -32602,
                &format!("Invalid primary key value: '{pk_val_owned}'."),
            )
        }
    };

    let sid = match spreadsheet_id(&id, params) {
        Ok(s) => s,
        Err(resp) => return resp,
    };

    let sheet_id = match get_sheet_id(&sid, &table) {
        Ok(n) => n,
        Err(e) => return error_response(id, -32000, &e.to_string()),
    };

    match delete_row(&sid, sheet_id, row_num) {
        Ok(()) => ok_response(id, json!(1)),
        Err(e) => error_response(id, -32000, &e.to_string()),
    }
}
