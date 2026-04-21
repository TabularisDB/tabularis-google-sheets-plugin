use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::access_token;

const SHEETS_BASE: &str = "https://sheets.googleapis.com/v4/spreadsheets";

// ---------------------------------------------------------------------------
// API response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct SpreadsheetMeta {
    sheets: Vec<SheetMeta>,
}

#[derive(Deserialize)]
struct SheetMeta {
    properties: SheetProperties,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SheetProperties {
    sheet_id: i64,
    title: String,
}

#[derive(Deserialize)]
struct ValuesResponse {
    #[serde(default)]
    values: Vec<Vec<Value>>,
}

// ---------------------------------------------------------------------------
// HTTP helper
// ---------------------------------------------------------------------------

fn get_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap()
}

/// Checks response status and returns a readable error that includes the Google API error message.
fn check_status(resp: reqwest::blocking::Response) -> anyhow::Result<reqwest::blocking::Response> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    let body = resp.text().unwrap_or_default();
    let detail = serde_json::from_str::<Value>(&body)
        .ok()
        .and_then(|v| {
            v["error"]["message"]
                .as_str()
                .map(str::to_owned)
        })
        .unwrap_or(body);
    anyhow::bail!("{status}: {detail}");
}

fn auth_request(
    client: &reqwest::blocking::Client,
    url: &str,
) -> anyhow::Result<reqwest::blocking::RequestBuilder> {
    let token = access_token(client)?;
    Ok(client.get(url).bearer_auth(token))
}

fn post_with_auth(
    client: &reqwest::blocking::Client,
    url: &str,
    body: Value,
) -> anyhow::Result<()> {
    let token = access_token(client)?;
    let resp = client.post(url).bearer_auth(&token).json(&body).send()?;
    check_status(resp)?;
    Ok(())
}

fn put_with_auth(
    client: &reqwest::blocking::Client,
    url: &str,
    body: Value,
) -> anyhow::Result<()> {
    let token = access_token(client)?;
    let resp = client.put(url).bearer_auth(&token).json(&body).send()?;
    check_status(resp)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Returns all sheet (tab) names.
pub fn get_sheet_names(spreadsheet_id: &str) -> anyhow::Result<Vec<String>> {
    let client = get_client();
    let url = format!("{SHEETS_BASE}/{spreadsheet_id}");
    let meta: SpreadsheetMeta = auth_request(&client, &url)?
        .send()
        .map_err(anyhow::Error::from)
        .and_then(check_status)?
        .json()?;
    Ok(meta.sheets.into_iter().map(|s| s.properties.title).collect())
}

/// Returns (sheet_id, sheet_names) map for all sheets.
pub fn get_sheet_id_map(spreadsheet_id: &str) -> anyhow::Result<Vec<(String, i64)>> {
    let client = get_client();
    let url = format!("{SHEETS_BASE}/{spreadsheet_id}");
    let meta: SpreadsheetMeta = auth_request(&client, &url)?
        .send()
        .map_err(anyhow::Error::from)
        .and_then(check_status)?
        .json()?;
    Ok(meta
        .sheets
        .into_iter()
        .map(|s| (s.properties.title, s.properties.sheet_id))
        .collect())
}

/// Returns numeric sheetId for a named sheet.
pub fn get_sheet_id(spreadsheet_id: &str, sheet_name: &str) -> anyhow::Result<i64> {
    let map = get_sheet_id_map(spreadsheet_id)?;
    map.into_iter()
        .find(|(name, _)| name == sheet_name)
        .map(|(_, id)| id)
        .ok_or_else(|| anyhow::anyhow!("Sheet '{}' not found.", sheet_name))
}

/// Returns (headers, data_rows). Row 0 = headers, rest = data.
/// All values are returned as serde_json::Value.
pub fn get_sheet_data(
    spreadsheet_id: &str,
    sheet_name: &str,
) -> anyhow::Result<(Vec<String>, Vec<Vec<Value>>)> {
    let client = get_client();
    let encoded = urlencoded(sheet_name);
    let url = format!(
        "{SHEETS_BASE}/{spreadsheet_id}/values/'{encoded}'!A:ZZ\
         ?valueRenderOption=UNFORMATTED_VALUE"
    );
    let resp: ValuesResponse = auth_request(&client, &url)?
        .send()
        .map_err(anyhow::Error::from)
        .and_then(check_status)?
        .json()?;

    if resp.values.is_empty() {
        return Ok((vec![], vec![]));
    }

    let headers: Vec<String> = resp.values[0]
        .iter()
        .map(|v| value_to_string(v))
        .collect();
    let data = resp.values[1..].to_vec();
    Ok((headers, data))
}

/// Appends a row to a sheet.
pub fn append_row(
    spreadsheet_id: &str,
    sheet_name: &str,
    row: Vec<String>,
) -> anyhow::Result<()> {
    let client = get_client();
    let encoded = urlencoded(sheet_name);
    let url = format!(
        "{SHEETS_BASE}/{spreadsheet_id}/values/'{encoded}'!A:A:append\
         ?valueInputOption=USER_ENTERED"
    );
    let body = json!({ "values": [row] });
    post_with_auth(&client, &url, body)?;
    Ok(())
}

/// Updates a single cell.
pub fn update_cell(
    spreadsheet_id: &str,
    sheet_name: &str,
    col_letter: &str,
    row_num: usize, // 1-indexed sheet row
    value: &str,
) -> anyhow::Result<()> {
    let client = get_client();
    let encoded = urlencoded(sheet_name);
    let url = format!(
        "{SHEETS_BASE}/{spreadsheet_id}/values/'{encoded}'!{col_letter}{row_num}\
         ?valueInputOption=USER_ENTERED"
    );
    let body = json!({ "values": [[value]] });
    put_with_auth(&client, &url, body)?;
    Ok(())
}

/// Deletes a row by its 1-indexed sheet row number.
pub fn delete_row(
    spreadsheet_id: &str,
    sheet_id: i64,
    row_num: usize, // 1-indexed sheet row
) -> anyhow::Result<()> {
    let client = get_client();
    let url = format!("{SHEETS_BASE}/{spreadsheet_id}:batchUpdate");
    let body = json!({
        "requests": [{
            "deleteDimension": {
                "range": {
                    "sheetId": sheet_id,
                    "dimension": "ROWS",
                    "startIndex": row_num - 1,
                    "endIndex": row_num
                }
            }
        }]
    });
    post_with_auth(&client, &url, body)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// Infer SQL type from a sample of values.
pub fn infer_type(values: &[Value]) -> &'static str {
    let mut has_float = false;
    for v in values {
        match v {
            Value::Null => continue,
            Value::String(s) if s.is_empty() => continue,
            Value::Bool(_) => return "TEXT",
            Value::Number(n) => {
                if n.as_f64().map(|f| f.fract() != 0.0).unwrap_or(false) {
                    has_float = true;
                }
            }
            Value::String(s) => {
                if s.parse::<i64>().is_ok() {
                    continue;
                }
                if s.parse::<f64>().is_ok() {
                    has_float = true;
                    continue;
                }
                return "TEXT";
            }
            _ => return "TEXT",
        }
    }
    if has_float {
        "REAL"
    } else {
        "INTEGER"
    }
}

fn urlencoded(s: &str) -> String {
    s.replace(' ', "%20").replace('\'', "%27")
}

/// Convert 1-based column index to A1 letter notation.
pub fn col_letter(mut n: usize) -> String {
    let mut result = String::new();
    while n > 0 {
        n -= 1;
        result.insert(0, char::from(b'A' + (n % 26) as u8));
        n /= 26;
    }
    result
}

/// Extract spreadsheet ID from an ID string or full URL.
pub fn extract_spreadsheet_id(input: &str) -> &str {
    if let Some(start) = input.find("/spreadsheets/d/") {
        let rest = &input[start + "/spreadsheets/d/".len()..];
        let end = rest.find('/').unwrap_or(rest.len());
        return &rest[..end];
    }
    input.trim()
}
