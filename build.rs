fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("resources/app.ico");
        res.set("ProductName", "BetterFolderSize");
        res.set("FileDescription", "Folder Size Scanner for Windows Explorer");
        res.set("LegalCopyright", "Copyright (c) 2026 UsableWindows");
        res.set("FileVersion", "0.1.0.0");
        res.set("ProductVersion", "0.1.0.0");
        if let Err(e) = res.compile() {
            eprintln!("Warning: failed to compile windows resource: {}", e);
        }
    }
}
