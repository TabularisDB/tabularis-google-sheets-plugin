pub mod crud;
pub mod ddl;
pub mod init;
pub mod metadata;
pub mod query;

use serde_json::Value;

use crate::rpc::error_response;
use crate::sheets::extract_spreadsheet_id;

/// Extract the spreadsheet id from `params.params.database`. Returns a
/// ready-made JSON-RPC error response (with the provided `id`) on failure.
pub fn spreadsheet_id(id: &Value, params: &Value) -> Result<String, Value> {
    let raw = params
        .get("params")
        .and_then(|p| p.get("database"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();

    if raw.is_empty() {
        return Err(error_response(
            id.clone(),
            -32602,
            "No Spreadsheet ID provided. Enter the Google Sheets Spreadsheet ID (or full URL) in the 'Database' field.",
        ));
    }
    Ok(extract_spreadsheet_id(raw).to_string())
}
