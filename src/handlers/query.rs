//! Connection + query execution for Google Sheets.
//!
//! `test_connection` actually hits the Sheets API with the saved token —
//! it's how we surface "you need to re-authorize" in the connection form.
//! `execute_query` parses a tiny SQL subset (see `crate::sql`) and turns
//! each statement into one or more Sheets REST calls.

use std::collections::HashMap;
use std::time::Instant;

use serde_json::{json, Value};

use crate::handlers::spreadsheet_id;
use crate::rpc::{error_response, ok_response};
use crate::sheets::{
    append_row, col_letter, delete_row, get_sheet_data, get_sheet_id, get_sheet_names, update_cell,
    value_to_string,
};
use crate::sql::{eval_where, extract_row_num, parse, Query};

pub fn test_connection(id: Value, params: &Value) -> Value {
    let sid = match spreadsheet_id(&id, params) {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    match get_sheet_names(&sid) {
        Ok(_) => ok_response(id, json!({ "success": true })),
        Err(e) => error_response(id, -32000, &e.to_string()),
    }
}

pub fn execute_query(id: Value, params: &Value) -> Value {
    let query = params
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let page = params.get("page").and_then(Value::as_u64).unwrap_or(1) as usize;
    let page_size = params
        .get("page_size")
        .and_then(Value::as_u64)
        .unwrap_or(100) as usize;

    let sid = match spreadsheet_id(&id, params) {
        Ok(s) => s,
        Err(resp) => return resp,
    };

    let parsed = match parse(query) {
        Ok(q) => q,
        Err(e) => return error_response(id, -32000, &e.to_string()),
    };

    let t0 = Instant::now();

    match parsed {
        Query::Select(sel) => run_select(id, &sid, sel, page, page_size, t0),
        Query::Insert(ins) => {
            let (headers, _) = match get_sheet_data(&sid, &ins.table) {
                Ok(v) => v,
                Err(e) => return error_response(id, -32000, &e.to_string()),
            };
            let row: Vec<String> = headers
                .iter()
                .map(|h| ins.data.get(h).cloned().unwrap_or_default())
                .collect();
            match append_row(&sid, &ins.table, row) {
                Ok(()) => ok_response(
                    id,
                    json!({
                        "columns": [],
                        "rows": [],
                        "affected_rows": 1,
                        "total_count": 0,
                        "execution_time_ms": t0.elapsed().as_millis() as u64
                    }),
                ),
                Err(e) => error_response(id, -32000, &e.to_string()),
            }
        }
        Query::Update(upd) => {
            let row_num = match extract_row_num(&upd.where_clause) {
                Ok(n) => n,
                Err(e) => return error_response(id, -32602, &e.to_string()),
            };
            let (headers, _) = match get_sheet_data(&sid, &upd.table) {
                Ok(v) => v,
                Err(e) => return error_response(id, -32000, &e.to_string()),
            };
            for (col, val) in &upd.set {
                if let Some(pos) = headers.iter().position(|h| h == col) {
                    let letter = col_letter(pos + 1);
                    if let Err(e) = update_cell(&sid, &upd.table, &letter, row_num, val) {
                        return error_response(id, -32000, &e.to_string());
                    }
                }
            }
            ok_response(
                id,
                json!({
                    "columns": [],
                    "rows": [],
                    "affected_rows": 1,
                    "total_count": 0,
                    "execution_time_ms": t0.elapsed().as_millis() as u64
                }),
            )
        }
        Query::Delete(del) => {
            let row_num = match extract_row_num(&del.where_clause) {
                Ok(n) => n,
                Err(e) => return error_response(id, -32602, &e.to_string()),
            };
            let sheet_id = match get_sheet_id(&sid, &del.table) {
                Ok(n) => n,
                Err(e) => return error_response(id, -32000, &e.to_string()),
            };
            match delete_row(&sid, sheet_id, row_num) {
                Ok(()) => ok_response(
                    id,
                    json!({
                        "columns": [],
                        "rows": [],
                        "affected_rows": 1,
                        "total_count": 0,
                        "execution_time_ms": t0.elapsed().as_millis() as u64
                    }),
                ),
                Err(e) => error_response(id, -32000, &e.to_string()),
            }
        }
    }
}

pub fn explain_query(id: Value, _params: &Value) -> Value {
    error_response(
        id,
        -32601,
        "Google Sheets does not support EXPLAIN — queries are flattened into REST API calls.",
    )
}

fn run_select(
    id: Value,
    sid: &str,
    sel: crate::sql::SelectQuery,
    page: usize,
    page_size: usize,
    t0: Instant,
) -> Value {
    let (headers, raw_rows) = match get_sheet_data(sid, &sel.table) {
        Ok(v) => v,
        Err(e) => return error_response(id, -32000, &e.to_string()),
    };

    if headers.is_empty() {
        return ok_response(
            id,
            json!({
                "columns": [],
                "rows": [],
                "affected_rows": 0,
                "total_count": 0,
                "execution_time_ms": 0
            }),
        );
    }

    let all_headers: Vec<String> = std::iter::once("_row".to_string())
        .chain(headers.iter().cloned())
        .collect();

    let mut rows: Vec<Vec<Value>> = raw_rows
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let row_num = i + 2; // sheet row (1=header, 2=first data row)
            let mut row = vec![json!(row_num)];
            for (j, _h) in headers.iter().enumerate() {
                row.push(r.get(j).cloned().unwrap_or(Value::Null));
            }
            row
        })
        .collect();

    if let Some(ref wc) = sel.where_clause {
        rows.retain(|row| {
            let row_map: HashMap<String, String> = all_headers
                .iter()
                .enumerate()
                .map(|(i, h)| {
                    (
                        h.clone(),
                        value_to_string(row.get(i).unwrap_or(&Value::Null)),
                    )
                })
                .collect();
            eval_where(wc, &row_map)
        });
    }

    let total_count = rows.len();

    if sel.count_only {
        return ok_response(
            id,
            json!({
                "columns": ["COUNT(*)"],
                "rows": [[total_count]],
                "affected_rows": 0,
                "total_count": 1,
                "execution_time_ms": t0.elapsed().as_millis() as u64
            }),
        );
    }

    let offset = sel.offset.unwrap_or_else(|| {
        if sel.limit.is_none() {
            (page - 1) * page_size
        } else {
            0
        }
    });
    let limit = sel.limit.unwrap_or(page_size);
    let paged: Vec<_> = rows.into_iter().skip(offset).take(limit).collect();

    let (result_cols, result_rows): (Vec<String>, Vec<Vec<Value>>) = match &sel.columns {
        None => (all_headers, paged),
        Some(selected) => {
            let indices: Vec<usize> = selected
                .iter()
                .map(|c| all_headers.iter().position(|h| h == c))
                .collect::<Option<Vec<_>>>()
                .unwrap_or_default();
            let proj_rows = paged
                .iter()
                .map(|row| {
                    indices
                        .iter()
                        .map(|&i| row.get(i).cloned().unwrap_or(Value::Null))
                        .collect()
                })
                .collect();
            (selected.clone(), proj_rows)
        }
    };

    ok_response(
        id,
        json!({
            "columns": result_cols,
            "rows": result_rows,
            "affected_rows": 0,
            "total_count": total_count,
            "execution_time_ms": t0.elapsed().as_millis() as u64
        }),
    )
}
