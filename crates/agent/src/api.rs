//! Local HTTP API (127.0.0.1 only) used by the UI and the browser extension.
//!
//! Protections against other websites talking to it:
//! * the `Host` header must be the loopback address (blocks DNS rebinding);
//! * cross-origin requests are only answered for browser-extension origins;
//! * every state-changing request needs the `X-FocusFort: 1` header, which a
//!   web page cannot send cross-origin without a CORS preflight we refuse.
//!
//! Lock rules are enforced in `App`, so even a local program using the API
//! cannot loosen a locked block.

use std::io::Read;
use std::sync::Arc;

use serde::Deserialize;
use serde_json::json;
use tiny_http::{Header, Method, Request, Response, Server};

use focusfort_core::model::{Config, LockKind};
use focusfort_core::protection::RequestKind;

use crate::app::{ApiError, ApiResult, App};

const MAX_BODY: u64 = 1024 * 1024;

static INDEX_HTML: &str = include_str!("../../../ui/index.html");
static APP_JS: &str = include_str!("../../../ui/app.js");
static CORE_JS: &str = include_str!("../../../ui/core.js");
static HOME_JS: &str = include_str!("../../../ui/home.js");
static EDITORS_JS: &str = include_str!("../../../ui/editors.js");
static PROTECTION_JS: &str = include_str!("../../../ui/protection.js");
static I18N_JS: &str = include_str!("../../../ui/i18n.js");
static STYLE_CSS: &str = include_str!("../../../ui/style.css");
static ICON_SVG: &str = include_str!("../../../ui/icon.svg");

pub fn serve(app: Arc<App>, port: u16, workers: usize) -> std::io::Result<Vec<std::thread::JoinHandle<()>>> {
    let server = Arc::new(Server::http(("127.0.0.1", port)).map_err(std::io::Error::other)?);
    let mut handles = Vec::new();
    for _ in 0..workers.max(1) {
        let server = server.clone();
        let app = app.clone();
        handles.push(std::thread::spawn(move || {
            for req in server.incoming_requests() {
                handle(&app, port, req);
            }
        }));
    }
    Ok(handles)
}

fn header<'a>(req: &'a Request, name: &str) -> Option<&'a str> {
    req.headers().iter().find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(name)).map(|h| h.value.as_str())
}

fn is_extension_origin(o: &str) -> bool {
    ["chrome-extension://", "moz-extension://", "extension://", "safari-web-extension://"].iter().any(|p| o.starts_with(p))
}

fn hdr(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("valid header")
}

fn handle(app: &App, port: u16, mut req: Request) {
    let host_ok = header(&req, "Host")
        .is_some_and(|h| h == format!("127.0.0.1:{port}") || h == format!("localhost:{port}"));
    let origin = header(&req, "Origin").map(str::to_string);
    let same_origin = |o: &str| o == format!("http://127.0.0.1:{port}") || o == format!("http://localhost:{port}");
    let origin_ok = origin.as_deref().is_none_or(|o| same_origin(o) || is_extension_origin(o));
    let cors_origin = origin.as_deref().filter(|o| is_extension_origin(o)).map(str::to_string);

    let respond = |req: Request, status: u16, content_type: &str, body: String| {
        let mut resp = Response::from_string(body)
            .with_status_code(status)
            .with_header(hdr("Content-Type", content_type))
            .with_header(hdr("Cache-Control", "no-store"))
            .with_header(hdr("X-Content-Type-Options", "nosniff"));
        if content_type.starts_with("text/html") {
            resp = resp
                .with_header(hdr("X-Frame-Options", "DENY"))
                .with_header(hdr("Content-Security-Policy", "default-src 'self'; style-src 'self'; script-src 'self'; frame-ancestors 'none'"));
        }
        if let Some(o) = &cors_origin {
            resp = resp
                .with_header(hdr("Access-Control-Allow-Origin", o))
                .with_header(hdr("Access-Control-Allow-Methods", "GET, POST, PUT, OPTIONS"))
                .with_header(hdr("Access-Control-Allow-Headers", "Content-Type, X-FocusFort"))
                .with_header(hdr("Vary", "Origin"));
        }
        let _ = req.respond(resp);
    };

    if !host_ok || !origin_ok {
        return respond(req, 403, "application/json", json!({ "code": "forbidden" }).to_string());
    }
    if req.method() == &Method::Options {
        return respond(req, 204, "text/plain", String::new());
    }

    let url = req.url().to_string();
    let (path, query) = url.split_once('?').unwrap_or((&url, ""));
    let path = path.to_string();
    let query = query.to_string();
    let method = req.method().clone();

    if method == Method::Get {
        let static_file = match path.as_str() {
            "/" | "/index.html" => Some(("text/html; charset=utf-8", INDEX_HTML)),
            "/app.js" => Some(("text/javascript; charset=utf-8", APP_JS)),
            "/core.js" => Some(("text/javascript; charset=utf-8", CORE_JS)),
            "/home.js" => Some(("text/javascript; charset=utf-8", HOME_JS)),
            "/editors.js" => Some(("text/javascript; charset=utf-8", EDITORS_JS)),
            "/protection.js" => Some(("text/javascript; charset=utf-8", PROTECTION_JS)),
            "/i18n.js" => Some(("text/javascript; charset=utf-8", I18N_JS)),
            "/style.css" => Some(("text/css; charset=utf-8", STYLE_CSS)),
            "/icon.svg" | "/favicon.ico" => Some(("image/svg+xml", ICON_SVG)),
            _ => None,
        };
        if let Some((ct, body)) = static_file {
            return respond(req, 200, ct, body.to_string());
        }
    } else if header(&req, "X-FocusFort") != Some("1") {
        return respond(req, 403, "application/json", json!({ "code": "missing_header" }).to_string());
    }

    let mut body = String::new();
    if method != Method::Get && req.as_reader().take(MAX_BODY).read_to_string(&mut body).is_err() {
        return respond(req, 400, "application/json", json!({ "code": "bad_body" }).to_string());
    }

    let result = route(app, &method, &path, &query, &body);
    let (status, value) = match result {
        Ok(v) => (200, v),
        Err(e) => (e.status, e.body),
    };
    respond(req, status, "application/json; charset=utf-8", value.to_string());
}

fn parse<T: for<'de> Deserialize<'de>>(body: &str) -> Result<T, ApiError> {
    serde_json::from_str(if body.trim().is_empty() { "{}" } else { body })
        .map_err(|e| ApiError::bad_request(format!("invalid JSON: {e}")))
}

fn query_param<'a>(query: &'a str, key: &str) -> Option<&'a str> {
    query.split('&').filter_map(|kv| kv.split_once('=')).find(|(k, _)| *k == key).map(|(_, v)| v)
}

#[derive(Deserialize)]
struct StartSession {
    block_list_id: String,
    minutes: i64,
    #[serde(default = "no_lock")]
    lock: LockKind,
}
fn no_lock() -> LockKind {
    LockKind::None
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Minutes {
    minutes: i64,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Answer {
    answer: Option<String>,
}

#[derive(Deserialize)]
struct NewChallenge {
    purpose: String,
}

#[derive(Deserialize)]
struct NewRequest {
    kind: RequestKind,
    #[serde(default)]
    reason: String,
}

#[derive(Deserialize)]
struct Activity {
    url: String,
    #[serde(default)]
    seconds: u64,
    #[serde(default)]
    opened: bool,
    #[serde(default)]
    browser: String,
}

fn family(name: &str) -> &'static str {
    if name.eq_ignore_ascii_case("firefox") {
        "firefox"
    } else {
        "chromium"
    }
}

fn route(app: &App, method: &Method, path: &str, query: &str, body: &str) -> ApiResult {
    let seg: Vec<&str> = path.trim_matches('/').split('/').collect();
    match (method, seg.as_slice()) {
        (Method::Get, ["api", "status"]) => app.status(),
        (Method::Put, ["api", "config"]) => app.update_config(parse::<Config>(body)?),
        (Method::Post, ["api", "presets", id]) => app.add_preset(id),
        (Method::Post, ["api", "practice", "end"]) => app.end_practice(),

        (Method::Post, ["api", "sessions"]) => {
            let b: StartSession = parse(body)?;
            app.start_session(&b.block_list_id, b.minutes, b.lock)
        }
        (Method::Post, ["api", "sessions", id, "extend"]) => app.extend_session(id, parse::<Minutes>(body)?.minutes),
        (Method::Post, ["api", "sessions", id, "stop"]) => {
            let b: Answer = parse(body)?;
            app.stop_session(id, b.answer.as_deref())
        }

        (Method::Post, ["api", "challenge"]) => app.new_challenge(&parse::<NewChallenge>(body)?.purpose),

        (Method::Post, ["api", "requests"]) => {
            let b: NewRequest = parse(body)?;
            app.create_request(b.kind, &b.reason)
        }
        (Method::Post, ["api", "requests", id, "challenge"]) => {
            let b: Answer = parse(body)?;
            app.solve_request_challenge(id, b.answer.as_deref().unwrap_or(""))
        }
        (Method::Post, ["api", "requests", id, "cancel"]) => app.cancel_request(id),
        (Method::Post, ["api", "requests", id, "apply"]) => app.apply_request(id),

        (Method::Get, ["api", "ext", "policy"]) => app.extension_policy(family(query_param(query, "browser").unwrap_or(""))),
        (Method::Post, ["api", "ext", "activity"]) => {
            let b: Activity = parse(body)?;
            app.extension_activity(family(&b.browser), &b.url, b.seconds, b.opened)
        }

        _ => Err(ApiError::not_found("route")),
    }
}

