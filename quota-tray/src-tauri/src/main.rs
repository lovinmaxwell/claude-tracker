#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    quota_tray_lib::run();
}
