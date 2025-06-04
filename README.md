# RustyCrashTool

RustyCrashTool is a small GUI utility that watches for application crashes on systems running systemd (such as Arch Linux). When a crash is detected, the tool saves a snapshot of recent journal entries and displays where it was stored.

## Usage

Run `cargo run --release` and keep the app open while you use your system. When an application crashes, a log snapshot will be saved to `~/.local/share/rusty_crash_tool/` and the path will be displayed in the window.
