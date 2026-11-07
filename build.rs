fn main() {
    println!("cargo:rerun-if-changed=assets/app-icon/icon.ico");
    println!("cargo:rerun-if-changed=assets/app-icon/windows.rc");
    if cfg!(target_os = "windows")
        && std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
    {
        let output =
            std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("nut75.res");
        let status = std::process::Command::new("rc.exe")
            .args(["/nologo", "/fo"])
            .arg(&output)
            .arg("assets/app-icon/windows.rc")
            .status()
            .expect("Windows SDK rc.exe is required to embed the NUT75 application icon");
        assert!(
            status.success(),
            "Could not compile Windows application icon"
        );
        println!("cargo:rustc-link-arg={}", output.display());
    }
}
