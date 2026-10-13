//! Embeds the icon, version information and manifest into the Windows executable.

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=assets/windows/doa6-trainer.exe.manifest");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let mut resource = winresource::WindowsResource::new();
    resource
        .set_icon("assets/icon.ico")
        .set_manifest_file("assets/windows/doa6-trainer.exe.manifest")
        .set("ProductName", "DOA6 Trainer +17")
        .set("FileDescription", "DOA6 Trainer +17")
        .set("InternalName", "doa6-trainer")
        .set("OriginalFilename", "doa6-trainer.exe")
        .set("CompanyName", "MauCastillo")
        .set("LegalCopyright", "© 2026 MauCastillo")
        .set("Comments", "Open-source trainer for Dead or Alive 6. Source: https://github.com/MauCastillo/doa6-trainer");
    resource.compile().expect("failed to embed Windows resources");
}
