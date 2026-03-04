use std::env;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=native/link_bridge.cpp");
    println!("cargo:rerun-if-changed=native/link_bridge.hpp");
    println!("cargo:rerun-if-changed=vendor/ableton-link/include/ableton/Link.hpp");
    println!("cargo:rerun-if-changed=vendor/ableton-link/modules/asio-standalone/asio/include");
    println!("cargo:rerun-if-env-changed=TREKR_BUILD_HASH");

    let build_hash = env::var("TREKR_BUILD_HASH")
        .unwrap_or_else(|_| git_short_hash().unwrap_or_else(|| "dev".to_string()));
    println!("cargo:rustc-env=TREKR_BUILD_HASH={build_hash}");

    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("target os available");
    let mut build = cc::Build::new();
    build.cpp(true);
    build.std("c++17");
    build.file("native/link_bridge.cpp");
    build.include("vendor/ableton-link/include");
    build.include("vendor/ableton-link/modules/asio-standalone/asio/include");

    if target_os == "windows" {
        build.define("LINK_PLATFORM_WINDOWS", "1");
        build.define("_SCL_SECURE_NO_WARNINGS", None);
        build.define("NOMINMAX", "1");
        build.flag("/EHsc");
        println!("cargo:rustc-link-lib=avrt");
        println!("cargo:rustc-link-lib=iphlpapi");
        println!("cargo:rustc-link-lib=ws2_32");
    } else if target_os == "macos" {
        build.define("LINK_PLATFORM_UNIX", "1");
        build.define("LINK_PLATFORM_MACOSX", "1");
    } else if target_os == "linux" {
        build.define("LINK_PLATFORM_UNIX", "1");
        build.define("LINK_PLATFORM_LINUX", "1");
        println!("cargo:rustc-link-lib=atomic");
        println!("cargo:rustc-link-lib=pthread");
    } else {
        panic!("unsupported target os for Ableton Link bridge: {target_os}");
    }

    build.compile("trekr_link_bridge");
}

fn git_short_hash() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let hash = String::from_utf8(output.stdout).ok()?;
    let hash = hash.trim();
    (!hash.is_empty()).then_some(hash.to_string())
}
