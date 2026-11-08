//! Local HID service for Glorious mice.
//!
//! ```text
//! glorious-service [--bind 127.0.0.1:25820] [--devices-json PATH] [--probe]
//! ```

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use tokio::net::TcpListener;
use tracing::info;

use glorious_service::api::{self, AppState, DEFAULT_BIND};
use glorious_service::devices::DeviceDatabase;
use glorious_service::manager::DeviceManager;
use glorious_service::ws;

fn main() -> ExitCode {
    let options = match Options::parse(std::env::args().skip(1)) {
        Ok(Some(options)) => options,
        Ok(None) => return ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}\n");
            print_help();
            return ExitCode::FAILURE;
        }
    };

    tracing_subscriber::fmt()
        .with_max_level(options.log_level())
        .with_target(false)
        .init();

    let (db, db_path) = match &options.devices_json {
        Some(path) => match DeviceDatabase::from_path(path) {
            Ok(db) => (db, path.clone()),
            Err(err) => {
                eprintln!("error: {err}");
                return ExitCode::FAILURE;
            }
        },
        None => match DeviceDatabase::load_default() {
            Ok((db, path)) => (db, path),
            Err(err) => {
                eprintln!("error: {err}");
                return ExitCode::FAILURE;
            }
        },
    };

    info!(
        devices = db.devices().len(),
        database = %db_path.display(),
        "loaded device database"
    );

    let manager = Arc::new(DeviceManager::new(db, options.delay));
    if options.probe {
        return probe(manager);
    }

    run_server(manager, options)
}

fn probe(manager: Arc<DeviceManager>) -> ExitCode {
    let devices = match manager.snapshot_detailed() {
        Ok(devices) => devices,
        Err(err) => {
            eprintln!("probe failed: {err}");
            return ExitCode::FAILURE;
        }
    };

    let report = serde_json::json!({
        "count": devices.len(),
        "devices": devices,
    });
    match serde_json::to_string_pretty(&report) {
        Ok(json) => println!("{json}"),
        Err(err) => {
            eprintln!("failed to encode probe report: {err}");
            return ExitCode::FAILURE;
        }
    }

    eprintln!();
    eprintln!("Detected {} Glorious device(s):", devices.len());
    for device in &devices {
        let firmware = device
            .firmware
            .as_ref()
            .map(|f| f.display.as_str())
            .unwrap_or("unknown");
        let battery = device
            .battery
            .as_ref()
            .map(|b| {
                format!(
                    "{}%{}",
                    b.percentage,
                    if b.charging { " (charging)" } else { "" }
                )
            })
            .unwrap_or_else(|| "n/a".to_string());
        let connection = device
            .connection
            .as_ref()
            .map(|c| format!("{} {}:{}", c.kind, c.vid, c.pid))
            .unwrap_or_else(|| "not connected".to_string());
        eprintln!(
            "  - {} ({}) [{}] fw={} battery={} status={:?}",
            device.name, device.slug, connection, firmware, battery, device.status
        );
    }

    if devices.is_empty() {
        eprintln!("No supported Glorious device is connected.");
        ExitCode::from(3)
    } else {
        ExitCode::SUCCESS
    }
}

fn run_server(manager: Arc<DeviceManager>, options: Options) -> ExitCode {
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(err) => {
            eprintln!("failed to start tokio runtime: {err}");
            return ExitCode::FAILURE;
        }
    };

    runtime.block_on(async move {
        let state = AppState::new(manager);
        ws::spawn_telemetry(state.clone());
        let app = api::router(state.clone());

        let listener = match TcpListener::bind(&options.bind).await {
            Ok(listener) => listener,
            Err(err) => {
                eprintln!("failed to bind {}: {err}", options.bind);
                return ExitCode::FAILURE;
            }
        };

        let address = listener
            .local_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| options.bind.clone());
        info!(%address, "glorious-service listening (HTTP + WebSocket /ws)");

        let host = if let Some(rest) = address.strip_prefix("0.0.0.0:") {
            format!("127.0.0.1:{rest}")
        } else {
            address.clone()
        };
        let url = format!("http://{host}");
        println!("Glorious Launcher web UI available at: {url}");

        let shutdown = Arc::new(tokio::sync::Notify::new());
        #[cfg(target_os = "linux")]
        if options.desktop {
            glorious_service::desktop::spawn(&state, url.clone(), shutdown.clone());
        }

        if options.open_browser {
            open_browser(&url);
        }

        match axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal(shutdown))
            .await
        {
            Ok(()) => {
                info!("server stopped");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("server error: {err}");
                ExitCode::FAILURE
            }
        }
    })
}

async fn shutdown_signal(shutdown: Arc<tokio::sync::Notify>) {
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = shutdown.notified() => {},
    }
    info!("shutdown signal received");
}

fn open_browser(url: &str) {
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(url)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(url)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .spawn();
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = url;
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Options {
    bind: String,
    devices_json: Option<PathBuf>,
    probe: bool,
    delay: Duration,
    verbose: bool,
    open_browser: bool,
    desktop: bool,
}

impl Options {
    fn parse<I: Iterator<Item = String>>(mut args: I) -> Result<Option<Self>, String> {
        let mut options = Options {
            bind: std::env::var("GLORIOUS_BIND").unwrap_or_else(|_| DEFAULT_BIND.to_string()),
            devices_json: std::env::var_os("GLORIOUS_DEVICES_JSON").map(PathBuf::from),
            probe: false,
            delay: default_delay(),
            verbose: false,
            open_browser: true,
            desktop: true,
        };

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--probe" => options.probe = true,
                "--open" => options.open_browser = true,
                "--no-browser" => options.open_browser = false,
                "--no-desktop" => options.desktop = false,
                "--verbose" | "-v" => options.verbose = true,
                "--bind" => {
                    options.bind = args
                        .next()
                        .ok_or_else(|| "--bind requires an address".to_string())?;
                }
                "--devices-json" => {
                    options.devices_json =
                        Some(PathBuf::from(args.next().ok_or_else(|| {
                            "--devices-json requires a path".to_string()
                        })?));
                }
                "--delay-ms" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--delay-ms requires a number".to_string())?;
                    options.delay = parse_delay(&value)?;
                }
                "-h" | "--help" => {
                    print_help();
                    return Ok(None);
                }
                "-V" | "--version" => {
                    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
                    return Ok(None);
                }
                other if other.starts_with("--bind=") => {
                    options.bind = other["--bind=".len()..].to_string();
                }
                other if other.starts_with("--devices-json=") => {
                    options.devices_json = Some(PathBuf::from(&other["--devices-json=".len()..]));
                }
                other if other.starts_with("--delay-ms=") => {
                    options.delay = parse_delay(&other["--delay-ms=".len()..])?;
                }
                other => return Err(format!("unknown argument '{other}'")),
            }
        }

        Ok(Some(options))
    }

    fn log_level(&self) -> tracing::Level {
        if self.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        }
    }
}

fn default_delay() -> Duration {
    std::env::var("GLORIOUS_HID_DELAY_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or(glorious_service::hid::DEFAULT_DELAY)
}

fn parse_delay(value: &str) -> Result<Duration, String> {
    let millis: u64 = value
        .parse()
        .map_err(|_| format!("invalid delay '{value}' (expected milliseconds)"))?;
    if millis > 2_000 {
        return Err("delay above 2000 ms is not useful".to_string());
    }
    Ok(Duration::from_millis(millis))
}

fn print_help() {
    println!(
        "\
glorious-service {version}
Local HID service + HTTP/WebSocket API for Glorious mice (Linux).

USAGE:
    glorious-service [OPTIONS]

OPTIONS:
    --bind <ADDR>            Listen address (default {bind}, env GLORIOUS_BIND)
    --devices-json <PATH>    Device database (default data/glorious_devices.json,
                             env GLORIOUS_DEVICES_JSON)
    --delay-ms <MS>          Delay between feature-report write/read (default 80,
                             env GLORIOUS_HID_DELAY_MS)
    --probe                  Read firmware/battery/settings once and print JSON
    --open                   Open browser on startup (default)
    --no-browser             Do not open browser on startup
    --no-desktop             Disable Linux system tray and notifications
    -v, --verbose            Debug logging
    -h, --help               Show this help
    -V, --version            Show version

HTTP API:
    GET  /api/health
    GET  /api/devices
    GET  /api/device/{{slug}}
    POST /api/device/{{slug}}/apply
    WS   /ws                     live telemetry (snapshot + every 3 s)

EXAMPLE:
    curl http://127.0.0.1:25820/api/devices
    curl http://127.0.0.1:25820/api/device/model-o-wireless
    curl -X POST http://127.0.0.1:25820/api/device/model-o-wireless/apply \\
         -H 'content-type: application/json' \\
         -d '{{\"dpi_stages\":[800,1600,3200],\"active_dpi_stage\":2,\"polling_rate\":1000}}'
",
        version = env!("CARGO_PKG_VERSION"),
        bind = DEFAULT_BIND,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_default_has_open_browser_enabled() {
        let opts = Options::parse(Vec::<String>::new().into_iter())
            .expect("parse succeeds")
            .expect("some options");
        assert!(opts.open_browser);
    }

    #[test]
    fn options_no_browser_flag_disables_browser() {
        let args = vec!["--no-browser".to_string()];
        let opts = Options::parse(args.into_iter())
            .expect("parse succeeds")
            .expect("some options");
        assert!(!opts.open_browser);
    }

    #[test]
    fn options_no_desktop_disables_desktop_integration() {
        let opts = Options::parse(["--no-desktop".to_string()].into_iter())
            .expect("parse succeeds")
            .expect("some options");
        assert!(!opts.desktop);
    }

    #[test]
    fn options_open_flag_enables_browser() {
        let args = vec!["--no-browser".to_string(), "--open".to_string()];
        let opts = Options::parse(args.into_iter())
            .expect("parse succeeds")
            .expect("some options");
        assert!(opts.open_browser);
    }
}
