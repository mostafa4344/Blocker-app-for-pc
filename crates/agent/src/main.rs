//! ContentGuard agent: the background service that enforces blocks.
//!
//! Usage:
//!   contentguard-agent run               run in the foreground (development, Linux/macOS service)
//!   contentguard-agent service           entry point used by the Windows service manager
//!   contentguard-agent uninstall-check   exit 0 if uninstalling is allowed right now
//!   contentguard-agent cleanup           remove ContentGuard entries from the hosts file
//!   contentguard-agent status            print the current status as JSON

mod api;
mod app;
mod enforce;
mod fingerprint;
mod hosts;
mod license;
mod paths;
mod processes;
mod safesearch;
#[cfg(windows)]
mod service;
mod sync;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use contentguard_core::store::Store;

use crate::app::App;

fn open_app() -> std::io::Result<Arc<App>> {
    let store = Store::new(paths::data_dir())?;
    Ok(Arc::new(App::open(store)?))
}

/// Starts the API, enforcement and sync threads; returns when `stop` is set.
pub fn run_until(stop: Arc<AtomicBool>) -> std::io::Result<()> {
    let app = open_app()?;
    let port = paths::listen_port();
    api::serve(app.clone(), port, 4)?;
    eprintln!(
        "contentguard: listening on http://127.0.0.1:{port} (data: {})",
        paths::data_dir().display()
    );
    let sync_app = app.clone();
    let sync_stop = stop.clone();
    std::thread::spawn(move || sync::run(sync_app, sync_stop));
    enforce::run(app, stop);
    Ok(())
}

/// Asks the running agent (so its in-memory state stays authoritative); falls
/// back to reading the data files when the agent is not running.
fn uninstall_check() -> std::io::Result<(bool, Arc<App>)> {
    let url = format!(
        "http://127.0.0.1:{}/api/uninstall/consume",
        paths::listen_port()
    );
    let http: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(5)))
        .http_status_as_error(false)
        .build()
        .into();
    let app = open_app()?;
    match http
        .post(&url)
        .header("X-ContentGuard", "1")
        .send_json(serde_json::json!({}))
    {
        Ok(mut resp) if resp.status() == 200 => {
            let v: serde_json::Value =
                resp.body_mut().read_json().map_err(std::io::Error::other)?;
            Ok((v["allowed"].as_bool().unwrap_or(false), app))
        }
        Ok(resp) => Err(std::io::Error::other(format!(
            "agent answered {}",
            resp.status()
        ))),
        Err(_) => Ok((app.uninstall_check()?, app)),
    }
}

fn main() {
    let cmd = std::env::args().nth(1).unwrap_or_else(|| "run".into());
    let code = match cmd.as_str() {
        "run" => {
            let stop = Arc::new(AtomicBool::new(false));
            match run_until(stop) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("contentguard: {e}");
                    1
                }
            }
        }
        #[cfg(windows)]
        "service" => match service::run() {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("contentguard: service error: {e}");
                1
            }
        },
        "uninstall-check" => match uninstall_check() {
            Ok((true, app)) => {
                sync::report_uninstalled(&app);
                println!("allowed");
                0
            }
            Ok((false, _)) => {
                println!("Uninstall protection is on. Open ContentGuard > Protection and request uninstall first.");
                2
            }
            Err(e) => {
                eprintln!("contentguard: {e}");
                1
            }
        },
        "cleanup" => match hosts::apply(&paths::hosts_file(), &[], &[]) {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("contentguard: {e}");
                1
            }
        },
        "status" => match open_app().map(|a| a.status()) {
            Ok(Ok(v)) => {
                println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
                0
            }
            Ok(Err(e)) => {
                eprintln!("contentguard: {:?}", e.body);
                1
            }
            Err(e) => {
                eprintln!("contentguard: {e}");
                1
            }
        },
        "version" | "--version" => {
            println!("contentguard-agent {}", app::VERSION);
            0
        }
        _ => {
            eprintln!("usage: contentguard-agent [run|uninstall-check|cleanup|status|version]");
            64
        }
    };
    std::process::exit(code);
}
