use std::env;

fn main() {
    println!("cargo:rerun-if-changed=native/link_bridge.cpp");
    println!("cargo:rerun-if-changed=native/link_bridge.hpp");
    println!("cargo:rerun-if-changed=vendor/ableton-link/include/ableton/Link.hpp");
    println!("cargo:rerun-if-changed=vendor/ableton-link/modules/asio-standalone/asio/include");

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
