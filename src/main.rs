use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use chrono::Local;
use directories::BaseDirs;
use eframe::egui;
use open as open_file;

/// Capture the last 100 lines from the system journal and save them to a file.
fn capture_snapshot() -> std::io::Result<String> {
    let output = Command::new("journalctl")
        .arg("-n")
        .arg("100")
        .output()?;

    let snapshot = String::from_utf8_lossy(&output.stdout);

    let base: PathBuf = BaseDirs::new()
        .map(|b| b.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("rusty_crash_tool");
    std::fs::create_dir_all(&dir)?;
    let file_path = dir.join(format!(
        "crash_{}.log",
        Local::now().format("%Y%m%d_%H%M%S")
    ));
    std::fs::write(&file_path, snapshot.as_bytes())?;
    Ok(file_path.to_string_lossy().into())
}

struct CrashToolApp {
    last_log_path: Arc<Mutex<Option<String>>>,
    _handle: thread::JoinHandle<()>,
}

impl CrashToolApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let last_log_path = Arc::new(Mutex::new(None));
        let thread_log = last_log_path.clone();

        // Spawn a background thread monitoring journalctl for crash messages
        let handle = thread::spawn(move || {
            let mut child = Command::new("journalctl")
                .arg("-f")
                .stdout(Stdio::piped())
                .spawn()
                .expect("failed to spawn journalctl -f");

            let stdout = child
                .stdout
                .take()
                .expect("failed to capture journalctl stdout");
            let reader = BufReader::new(stdout);

            for line in reader.lines() {
                let line = match line {
                    Ok(l) => l,
                    Err(_) => break,
                };

                let lower = line.to_lowercase();
                if lower.contains("segmentation fault") || lower.contains("core dumped") {
                    if let Ok(path) = capture_snapshot() {
                        *thread_log.lock().unwrap() = Some(path);
                    }
                }
            }

            // Ensure the child process is terminated when the thread exits
            let _ = child.kill();
        });

        Self {
            last_log_path,
            _handle: handle,
        }
    }
}

impl eframe::App for CrashToolApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Rusty Crash Tool");
            ui.label("Monitoring system logs for crashes...");

            let maybe_path = self.last_log_path.lock().unwrap();
            if let Some(path) = maybe_path.as_ref() {
                ui.separator();
                ui.label("Last crash log saved to:");
                ui.horizontal(|ui| {
                    ui.monospace(path);
                    if ui.button("Open").clicked() {
                        let _ = open_file::that(path);
                    }
                    if ui.button("Copy").clicked() {
                        ui.ctx().copy_text(path.clone());
                    }
                });
            } else {
                ui.separator();
                ui.label("No crashes detected yet.");
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rusty Crash Tool",
        options,
        Box::new(|cc| Ok(Box::new(CrashToolApp::new(cc)))),
    )
}

