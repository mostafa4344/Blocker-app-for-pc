//! Windows service entry point. The installer registers the service as
//! `contentguard-agent.exe service` with automatic start.

use std::ffi::OsString;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use windows_service::service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
use windows_service::{define_windows_service, service_dispatcher};

pub const SERVICE_NAME: &str = "ContentGuard";

define_windows_service!(ffi_service_main, service_main);

pub fn run() -> windows_service::Result<()> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)
}

fn service_main(_args: Vec<OsString>) {
    if let Err(e) = run_service() {
        eprintln!("contentguard: {e}");
    }
}

fn run_service() -> windows_service::Result<()> {
    let stop = Arc::new(AtomicBool::new(false));
    let handler_stop = stop.clone();
    let status_handle =
        service_control_handler::register(SERVICE_NAME, move |event| match event {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                handler_stop.store(true, Ordering::Relaxed);
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        })?;

    let status = |state, accept| ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: state,
        controls_accepted: accept,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(5),
        process_id: None,
    };
    let accept = |allow_stop: bool| {
        if allow_stop {
            ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN
        } else {
            ServiceControlAccept::SHUTDOWN
        }
    };
    let mut allow_stop = crate::ALLOW_STOP.load(Ordering::Relaxed);
    status_handle.set_service_status(status(ServiceState::Running, accept(allow_stop)))?;
    let run_stop = stop.clone();
    let worker = std::thread::spawn(move || crate::run_until(run_stop));
    // Keep the accepted controls in sync with the protection state.
    while !stop.load(Ordering::Relaxed) && !worker.is_finished() {
        std::thread::sleep(Duration::from_millis(500));
        let now_allowed = crate::ALLOW_STOP.load(Ordering::Relaxed);
        if now_allowed != allow_stop {
            allow_stop = now_allowed;
            let _ =
                status_handle.set_service_status(status(ServiceState::Running, accept(allow_stop)));
        }
    }
    stop.store(true, Ordering::Relaxed);
    let failed = matches!(worker.join(), Ok(Err(_)) | Err(_));
    if failed {
        // Report a failure so the service recovery actions restart us.
        let mut st = status(ServiceState::Stopped, ServiceControlAccept::empty());
        st.exit_code = ServiceExitCode::Win32(1067);
        let _ = status_handle.set_service_status(st);
        return Ok(());
    }
    status_handle
        .set_service_status(status(ServiceState::Stopped, ServiceControlAccept::empty()))?;
    Ok(())
}
