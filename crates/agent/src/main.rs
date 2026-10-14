//! FocusFort agent: the background service that enforces blocks.
//!
//! Usage:
//!   focusfort-agent run               run in the foreground (development, Linux/macOS service)
//!   focusfort-agent service           entry point used by the Windows service manager
//!   focusfort-agent uninstall-check   exit 0 if uninstalling is allowed right now
//!   focusfort-agent cleanup           remove FocusFort entries from the hosts file
//!   focusfort-agent status            print the current status as JSON

mod api;
mod app;
mod enforce;
mod hosts;
mod paths;
mod processes;
#[cfg(windows)]
mod service;
mod sync;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use focusfort_core::store::Store;

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
    eprintln!("focusfort: listening on http://127.0.0.1:{port} (data: {})", paths::data_dir().display());
    let sync_app = app.clone();
    let sync_stop = stop.clone();
    std::thread::spawn(move || sync::run(sync_app, sync_stop));
    enforce::run(app, stop);
    Ok(())
}

fn main() {
    let cmd = std::env::args().nth(1).unwrap_or_else(|| "run".into());
    let code = match cmd.as_str() {
        "run" => {
            let stop = Arc::new(AtomicBool::new(false));
            match run_until(stop) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("focusfort: {e}");
                    1
                }
            }
        }
        #[cfg(windows)]
        "service" => match service::run() {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("focusfort: service error: {e}");
                1
            }
        },
        "uninstall-check" => match open_app().and_then(|app| Ok((app.uninstall_check()?, app))) {
            Ok((true, app)) => {
                sync::report_uninstalled(&app);
                println!("allowed");
                0
            }
            Ok((false, _)) => {
                println!("Uninstall protection is on. Open FocusFort > Protection and request uninstall first.");
                2
            }
            Err(e) => {
                eprintln!("focusfort: {e}");
                1
            }
        },
        "cleanup" => match hosts::apply(&paths::hosts_file(), &[]) {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("focusfort: {e}");
                1
            }
        },
        "status" => match open_app().map(|a| a.status()) {
            Ok(Ok(v)) => {
                println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
                0
            }
            Ok(Err(e)) => {
                eprintln!("focusfort: {:?}", e.body);
                1
            }
            Err(e) => {
                eprintln!("focusfort: {e}");
                1
            }
        },
        "version" | "--version" => {
            println!("focusfort-agent {}", app::VERSION);
            0
        }
        _ => {
            eprintln!("usage: focusfort-agent [run|uninstall-check|cleanup|status|version]");
            64
        }
    };
    std::process::exit(code);
}
