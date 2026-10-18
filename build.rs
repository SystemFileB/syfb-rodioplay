use winresource::WindowsResource;
use std::env;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        println!("cargo:rerun-if-env-changed=WINDRES");
        let mut res=WindowsResource::new();
        if let Ok(windres_path)=env::var("WINDRES") {
            res.set_windres_path(&windres_path);
        }
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
}

