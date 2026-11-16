fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "Mari Encoder");
        res.set("FileDescription", "Mari Encoder");
        res.set("CompanyName", "Flávio Takemoto");
        res.set("LegalCopyright", "© 2026 Flávio Takemoto — GPL-3.0-or-later");
        res.set("OriginalFilename", "MariEncoder.exe");
        if let Err(e) = res.compile() {
            println!("cargo:warning=não foi possível embutir o ícone: {e}");
        }
    }
}
