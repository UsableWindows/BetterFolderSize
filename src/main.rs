#![windows_subsystem = "windows"]

mod format;
mod scanner;
mod ui;

use std::env;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        native_windows_gui::init().ok();
        native_windows_gui::simple_message(
            "BetterFolderSize v0.1",
            "Usage:\nRight-click any folder in Windows Explorer and select:\n\"Folder Size (Better)\"\n\nOr run from the command line:\nBetterFolderSize.exe <folder-path>",
        );
        return;
    }

    let raw_path = &args[1];
    let path = PathBuf::from(raw_path.trim_matches('"'));

    if !path.exists() {
        native_windows_gui::init().ok();
        native_windows_gui::error_message(
            "BetterFolderSize - Error",
            &format!("The specified path does not exist or is inaccessible:\n{}", path.display()),
        );
        return;
    }

    ui::FolderSizeApp::run(&path);
}
