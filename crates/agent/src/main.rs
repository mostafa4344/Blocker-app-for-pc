//! ContentGuard agent: the background service that enforces blocks.
//!
//! Usage:
//!   contentguard-agent run               run in the foreground (development, Linux/macOS service)
//!   contentguard-agent service           entry point used by the Windows service manager
//!   contentguard-agent uninstall-check   exit 0 if uninstalling is allowed right now
//!   contentguard-agent cleanup           remove ContentGuard entries from the hosts file
//!   contentguard-agent status            print the current status as JSON
//!   contentguard-agent presets-json      print the built-in ready-made lists

mod api;
mod app;
mod apps;
mod browserpolicy;
mod dns;
mod enforce;
mod fingerprint;
mod hosts;
mod license;
mod notify;
mod paths;
mod processes;
mod safesearch;
#[cfg(windows)]
mod service;
mod sync;
#[cfg(feature = "tls")]
mod update;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// Whether the OS service may currently be stopped. While protection is on
/// (and no approved uninstall is in progress) the Windows service refuses the
/// Stop control so blocking cannot be switched off with `sc stop`. Updated by
/// the enforcement loop; starts false so a protected install is safe at boot.
pub static ALLOW_STOP: AtomicBool = AtomicBool::new(false);

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

/// Last crash, kept in the data dir and reported once (anonymously) with the
/// next license check, then deleted.
pub const CRASH_FILE: &str = "crash.log";

/// Records panics (message + source location only) so crashes on users' PCs
/// can be found and fixed.
fn install_crash_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let msg = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_default();
        let at = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_default();
        let line = format!(
            "{} v{} panic at {at}: {}",
            chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ"),
            app::VERSION,
            msg.chars().take(300).collect::<String>()
        );
        let _ = std::fs::write(paths::data_dir().join(CRASH_FILE), line);
        default(info);
    }));
}

fn main() {
    install_crash_hook();
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
            Ok(_) => {
                browserpolicy::remove();
                browserpolicy::set_private_windows_blocked(false);
                browserpolicy::set_secure_dns_off(false);
                browserpolicy::set_safe_mode_start(false);
                0
            }
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
        // Publisher tools for signed automatic updates (see docs/AUTO-UPDATE.md).
        #[cfg(feature = "tls")]
        "update-keygen" => match std::env::args().nth(2) {
            Some(out) => match update::keygen(std::path::Path::new(&out)) {
                Ok(public) => {
                    println!("private key written to {out} (keep it secret, back it up)");
                    println!("public key: {public}");
                    0
                }
                Err(e) => {
                    eprintln!("contentguard: {e}");
                    1
                }
            },
            None => {
                eprintln!("usage: contentguard-agent update-keygen <private-key-file>");
                64
            }
        },
        #[cfg(feature = "tls")]
        "update-sign" => {
            let a: Vec<String> = std::env::args().skip(2).collect();
            if a.len() != 3 {
                eprintln!("usage: contentguard-agent update-sign <installer.exe> <version> <private-key-file>");
                64
            } else {
                match update::sign(
                    std::path::Path::new(&a[0]),
                    &a[1],
                    std::path::Path::new(&a[2]),
                ) {
                    Ok(r) => {
                        println!("{}", serde_json::to_string_pretty(&r).unwrap_or_default());
                        0
                    }
                    Err(e) => {
                        eprintln!("contentguard: {e}");
                        1
                    }
                }
            }
        }
        // Prints the built-in ready-made lists (used to seed the server table).
        "presets-json" => {
            let defs = contentguard_core::presets::builtin_defs();
            println!(
                "{}",
                serde_json::to_string_pretty(&defs).unwrap_or_default()
            );
            0
        }
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
