use crate::channels::{get_channel_logs, get_channels_json, get_stream_logs, get_streams_json};
use crate::output::FunctionsJson;
use crate::{FunctionLogsJson, QueryRequest, HOTPATH_STATE};
use crossbeam_channel::bounded;
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::{LazyLock, OnceLock};
use std::thread;
use std::time::Duration;
use tiny_http::{Header, Request, Response, Server};

static RE_CHANNEL_LOGS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^/channels/(\d+)/logs$").unwrap());
static RE_STREAM_LOGS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^/streams/(\d+)/logs$").unwrap());
static RE_FUNCTION_LOGS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^/functions/([^/]+)/logs$").unwrap());

/// Tracks whether the HTTP server has been started to prevent duplicate instances
static HTTP_SERVER_STARTED: OnceLock<()> = OnceLock::new();

/// Starts the HTTP metrics server if it hasn't been started yet.
/// Uses OnceLock to ensure only one server instance is created.
pub fn start_metrics_server_once(port: u16) {
    HTTP_SERVER_STARTED.get_or_init(|| {
        start_metrics_server(port);
    });
}

fn start_metrics_server(port: u16) {
    thread::Builder::new()
        .name("hotpath-http-server".into())
        .spawn(move || {
            let addr = format!("0.0.0.0:{}", port);
            let server = match Server::http(&addr) {
                Ok(s) => s,
                Err(e) => {
                    panic!(
                        "Failed to bind metrics server to {}: {}. Customize the port using the HOTPATH_HTTP_PORT environment variable.",
                        addr, e
                    );
                }
            };

            eprintln!("[hotpath] Metrics server listening on http://{}", addr);

            for request in server.incoming_requests() {
                handle_request(request);
            }
        })
        .expect("Failed to spawn HTTP metrics server thread");
}

fn handle_request(request: Request) {
    let path = request.url().split('?').next().unwrap_or("/").to_string();

    match path.as_str() {
        "/metrics" => {
            let metrics = get_functions_json();
            respond_json(request, &metrics);
        }
        "/channels" => {
            let channels = get_channels_json();
            respond_json(request, &channels);
        }
        "/streams" => {
            let streams = get_streams_json();
            respond_json(request, &streams);
        }
        _ => {
            // Handle /functions/<encoded_key>/logs
            if let Some(caps) = RE_FUNCTION_LOGS.captures(&path) {
                handle_function_logs_request(request, &caps[1]);
                return;
            }

            // Handle /channels/<id>/logs
            if let Some(caps) = RE_CHANNEL_LOGS.captures(&path) {
                match get_channel_logs(&caps[1]) {
                    Some(logs) => respond_json(request, &logs),
                    None => respond_error(request, 404, "Channel not found"),
                }
                return;
            }

            // Handle /streams/<id>/logs
            if let Some(caps) = RE_STREAM_LOGS.captures(&path) {
                match get_stream_logs(&caps[1]) {
                    Some(logs) => respond_json(request, &logs),
                    None => respond_error(request, 404, "Stream not found"),
                }
                return;
            }

            respond_error(request, 404, "Not found");
        }
    }
}

fn respond_json<T: Serialize>(request: Request, value: &T) {
    match serde_json::to_vec(value) {
        Ok(body) => {
            let mut response = Response::from_data(body);
            response.add_header(
                Header::from_bytes(b"Content-Type".as_slice(), b"application/json".as_slice())
                    .unwrap(),
            );
            let _ = request.respond(response);
        }
        Err(e) => respond_internal_error(request, e),
    }
}

fn respond_error(request: Request, code: u16, msg: &str) {
    let _ = request.respond(Response::from_string(msg).with_status_code(code));
}

fn respond_internal_error(request: Request, e: impl Display) {
    eprintln!("Internal server error: {}", e);
    let _ = request.respond(
        Response::from_string(format!("Internal server error: {}", e)).with_status_code(500),
    );
}

fn handle_function_logs_request(request: Request, encoded_key: &str) {
    let function_name = match base64_decode(encoded_key) {
        Ok(name) => name,
        Err(e) => {
            respond_error(request, 400, &format!("Invalid base64 encoding: {}", e));
            return;
        }
    };

    // Get logs from worker thread
    match get_function_logs(&function_name) {
        Some(function_logs_json) => {
            respond_json(request, &function_logs_json);
        }
        None => {
            respond_error(
                request,
                404,
                &format!(
                    "Function '{}' not found or no logs available",
                    function_name
                ),
            );
        }
    }
}

fn base64_decode(encoded: &str) -> Result<String, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| e.to_string())?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

fn get_function_logs(function_name: &str) -> Option<FunctionLogsJson> {
    let arc_swap = HOTPATH_STATE.get()?;
    let state_option = arc_swap.load();
    let state_arc = (*state_option).as_ref()?.clone();

    let state_guard = state_arc.read().ok()?;

    let (response_tx, response_rx) = bounded::<Option<FunctionLogsJson>>(1);

    if let Some(query_tx) = &state_guard.query_tx {
        query_tx
            .send(QueryRequest::GetFunctionCalls {
                function_name: function_name.to_string(),
                response_tx,
            })
            .ok()?;
        drop(state_guard);

        // Receive the response - it will be Some(FunctionLogsJson) or None
        response_rx
            .recv_timeout(Duration::from_millis(250))
            .ok()
            .flatten()
    } else {
        None
    }
}

fn get_functions_json() -> FunctionsJson {
    if let Some(metrics) = try_get_functions_from_worker() {
        return metrics;
    }

    // Fallback if query fails: return empty functions data
    FunctionsJson {
        hotpath_profiling_mode: crate::output::ProfilingMode::Timing,
        total_elapsed: 0,
        description: "No functions data available yet".to_string(),
        caller_name: "hotpath".to_string(),
        percentiles: vec![95],
        data: crate::output::FunctionsDataJson(HashMap::new()),
    }
}

fn try_get_functions_from_worker() -> Option<FunctionsJson> {
    let arc_swap = HOTPATH_STATE.get()?;
    let state_option = arc_swap.load();
    let state_arc = (*state_option).as_ref()?.clone();

    let state_guard = state_arc.read().ok()?;

    let (response_tx, response_rx) = bounded::<FunctionsJson>(1);

    if let Some(query_tx) = &state_guard.query_tx {
        query_tx
            .send(QueryRequest::GetFunctions(response_tx))
            .ok()?;
        drop(state_guard);

        response_rx.recv_timeout(Duration::from_millis(250)).ok()
    } else {
        None
    }
}
