#[cfg(windows)]
use winresource::WindowsResource;

#[cfg(windows)]
fn main() {
    let mut res=WindowsResource::new();
    res.set("FileVersion",env!("CARGO_PKG_VERSION"));
    res.set("ProductVersion",env!("CARGO_PKG_VERSION"));
    res.set("ProductName","SyFB's rodioplay");
    res.set("FileDescription","SyFB's rodioplay");
    res.set("CompanyName","SystemFileB");
    res.set("LegalCopyright","Copyright (C) SystemFileB, Licensed under MPL 2.0");
    res.set("OriginalFilename","rodioplay.exe");
    res.set("InternalName","rodioplay");

    res.compile().unwrap();
}

#[cfg(not(windows))]
fn main() {}
