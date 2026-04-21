//! Entry point: read JSON-RPC lines from stdin, dispatch, write responses.

use std::io::{self, BufRead, Write};

mod auth;
mod client;
mod error;
mod handlers;
mod models;
mod rpc;
mod sheets;
mod sql;
mod utils;

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let response = rpc::handle_line(trimmed);
        let mut body = match serde_json::to_string(&response) {
            Ok(s) => s,
            Err(err) => format!(
                "{{\"jsonrpc\":\"2.0\",\"error\":{{\"code\":-32603,\"message\":\"serialization failed: {err}\"}},\"id\":null}}",
            ),
        };
        body.push('\n');
        if out.write_all(body.as_bytes()).is_err() {
            break;
        }
        let _ = out.flush();
    }
}
