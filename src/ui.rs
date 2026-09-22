extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use crate::format::{format_number, format_size_full};
use crate::scanner::{scan_directory, ScanStats};
use nwd::NwgUi;
use nwg::NativeUi;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

/// UI structure for BetterFolderSize
#[derive(Default, NwgUi)]
pub struct FolderSizeApp {
    // Shared state
    folder_path: RefCell<PathBuf>,
    cancel_flag: RefCell<Option<Arc<AtomicBool>>>,
    scan_result: RefCell<Option<Arc<Mutex<Option<ScanStats>>>>>,
    worker_handle: RefCell<Option<thread::JoinHandle<()>>>,

    // Window
    #[nwg_control(
        size: (440, 220),
        title: "Folder Size (Better)",
        flags: "WINDOW|VISIBLE"
    )]
    #[nwg_events(
        OnWindowClose: [FolderSizeApp::on_close],
        OnKeyEsc: [FolderSizeApp::on_close]
    )]
    window: nwg::Window,

    // Cross-thread notice
    #[nwg_control]
    #[nwg_events(OnNotice: [FolderSizeApp::on_scan_completed])]
    notice: nwg::Notice,

    // Fonts
    #[nwg_resource(family: "Segoe UI", size: 25, weight: 700)]
    font_large: nwg::Font,

    #[nwg_resource(family: "Segoe UI", size: 16, weight: 600)]
    font_bold: nwg::Font,

    #[nwg_resource(family: "Segoe UI", size: 14)]
    font_normal: nwg::Font,

    #[nwg_resource(family: "Segoe UI", size: 12)]
    font_subtle: nwg::Font,

    // Labels: Folder Header
    #[nwg_control(text: "Folder:", position: (20, 14), size: (400, 22), font: Some(&data.font_bold))]
    title_label: nwg::Label,

    #[nwg_control(text: "", position: (20, 38), size: (400, 18), font: Some(&data.font_subtle))]
    path_label: nwg::Label,

    // Scanning status indicator
    #[nwg_control(
        text: "Calculating folder size...",
        position: (20, 68),
        size: (400, 24),
        font: Some(&data.font_normal)
    )]
    status_label: nwg::Label,

    // Progress bar (animated marquee while scanning)
    #[nwg_control(
        position: (20, 96),
        size: (400, 14),
        flags: "VISIBLE|MARQUEE",
        marquee: true
    )]
    progress_bar: nwg::ProgressBar,

    // Result size (large bold font, shown when scan is finished)
    #[nwg_control(
        text: "",
        position: (20, 64),
        size: (400, 36),
        font: Some(&data.font_large)
    )]
    size_label: nwg::Label,

    // Details: files & folders
    #[nwg_control(text: "", position: (20, 106), size: (400, 22), font: Some(&data.font_normal))]
    details_label: nwg::Label,

    // Skipped items warning (if any)
    #[nwg_control(text: "", position: (20, 132), size: (400, 18), font: Some(&data.font_subtle))]
    skipped_label: nwg::Label,

    // Action buttons
    #[nwg_control(text: "Copy", position: (20, 164), size: (100, 32), font: Some(&data.font_normal))]
    #[nwg_events(OnButtonClick: [FolderSizeApp::on_copy])]
    copy_button: nwg::Button,

    #[nwg_control(text: "", position: (130, 170), size: (170, 20), font: Some(&data.font_subtle))]
    copied_label: nwg::Label,

    #[nwg_control(text: "Close", position: (320, 164), size: (100, 32), font: Some(&data.font_normal), focus: true)]
    #[nwg_events(OnButtonClick: [FolderSizeApp::on_close])]
    close_button: nwg::Button,
}

impl FolderSizeApp {
    /// Launches the UI and begins background folder scan
    pub fn run(folder: &Path) {
        nwg::init().expect("Failed to initialize Native Windows GUI");
        nwg::Font::set_global_family("Segoe UI").ok();

        let app = FolderSizeApp::build_ui(Default::default())
            .expect("Failed to create application window");

        // Center window on primary monitor
        let (sw, sh) = (nwg::Monitor::width(), nwg::Monitor::height());
        let (w, h) = (440, 220);
        let x = (sw - w) / 2;
        let y = (sh - h) / 2;
        app.window.set_position(x, y);

        // Set embedded application icon on window titlebar
        unsafe {
            use winapi::um::libloaderapi::GetModuleHandleW;
            use winapi::um::winuser::{LoadIconW, SendMessageW, ICON_BIG, ICON_SMALL, MAKEINTRESOURCEW, WM_SETICON};

            let h_instance = GetModuleHandleW(std::ptr::null());
            let h_icon = LoadIconW(h_instance, MAKEINTRESOURCEW(1));
            if !h_icon.is_null() {
                if let Some(hwnd) = app.window.handle.hwnd() {
                    SendMessageW(hwnd, WM_SETICON, ICON_BIG as _, h_icon as _);
                    SendMessageW(hwnd, WM_SETICON, ICON_SMALL as _, h_icon as _);
                }
            }
        }

        // Initially hide result labels
        app.size_label.set_visible(false);
        app.details_label.set_visible(false);
        app.skipped_label.set_visible(false);

        // Store folder path
        *app.folder_path.borrow_mut() = folder.to_path_buf();

        // Update header labels
        let folder_name = folder
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| folder.to_string_lossy().to_string());
        app.title_label.set_text(&format!("Folder: {}", folder_name));

        let full_path_str = folder.to_string_lossy().to_string();
        let display_path = if full_path_str.len() > 55 {
            format!("...{}", &full_path_str[full_path_str.len() - 52..])
        } else {
            full_path_str
        };
        app.path_label.set_text(&display_path);

        // Start background scanner
        let cancel = Arc::new(AtomicBool::new(false));
        *app.cancel_flag.borrow_mut() = Some(cancel.clone());

        let result_container = Arc::new(Mutex::new(None));
        *app.scan_result.borrow_mut() = Some(result_container.clone());

        let notice_sender = app.notice.sender();
        let path_to_scan = folder.to_path_buf();

        let handle = thread::spawn(move || {
            let stats = scan_directory(&path_to_scan, &cancel);
            if !cancel.load(Ordering::Relaxed) {
                if let Ok(mut lock) = result_container.lock() {
                    *lock = Some(stats);
                }
                notice_sender.notice();
            }
        });

        *app.worker_handle.borrow_mut() = Some(handle);

        nwg::dispatch_thread_events();
    }

    /// Invoked on UI thread when background scan finishes
    fn on_scan_completed(&self) {
        // Hide progress indicators
        self.status_label.set_visible(false);
        self.progress_bar.set_visible(false);

        let stats_opt = self
            .scan_result
            .borrow()
            .as_ref()
            .and_then(|r| r.lock().ok().and_then(|lock| lock.clone()));

        if let Some(stats) = stats_opt {
            let size_text = format_size_full(stats.total_bytes);
            self.size_label.set_text(&size_text);
            self.size_label.set_visible(true);

            let details_text = format!(
                "Files: {}    •    Folders: {}",
                format_number(stats.file_count),
                format_number(stats.dir_count)
            );
            self.details_label.set_text(&details_text);
            self.details_label.set_visible(true);

            if stats.skipped_count > 0 {
                self.skipped_label.set_text(&format!(
                    "(skipped {} inaccessible items)",
                    format_number(stats.skipped_count)
                ));
                self.skipped_label.set_visible(true);
            }
        }
    }

    /// Copies formatted result to clipboard
    fn on_copy(&self) {
        let path = self.folder_path.borrow();
        let stats_opt = self
            .scan_result
            .borrow()
            .as_ref()
            .and_then(|r| r.lock().ok().and_then(|lock| lock.clone()));

        let text = match stats_opt {
            Some(stats) => {
                let mut s = format!(
                    "Folder: {}\r\nSize: {}\r\nContent: {} files, {} folders",
                    path.display(),
                    format_size_full(stats.total_bytes),
                    format_number(stats.file_count),
                    format_number(stats.dir_count)
                );
                if stats.skipped_count > 0 {
                    s.push_str(&format!(
                        "\r\n(skipped {} inaccessible items)",
                        format_number(stats.skipped_count)
                    ));
                }
                s
            }
            None => format!("Folder: {}\r\nCalculating folder size in progress...", path.display()),
        };

        nwg::Clipboard::set_data_text(&self.window.handle, &text);
        self.copied_label.set_text("✓ Copied!");
    }

    /// Closes the application
    fn on_close(&self) {
        // Signal cancellation to background worker
        if let Some(cancel) = self.cancel_flag.borrow().as_ref() {
            cancel.store(true, Ordering::Relaxed);
        }
        nwg::stop_thread_dispatch();
    }
}
