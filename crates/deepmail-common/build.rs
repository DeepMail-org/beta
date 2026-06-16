use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to link the Zig library
    let zig_lib_path = PathBuf::from("/home/jain/Study/project/backend/zig-modules/rate-limiter/zig-out/lib");
    
    println!("cargo:rustc-link-search=native={}", zig_lib_path.display());
    println!("cargo:rustc-link-lib=zig_ratelimiter");
    
    // Rebuild if Zig source changes
    println!("cargo:rerun-if-changed=../../../zig-modules/rate-limiter/src/main.zig");
    println!("cargo:rerun-if-changed=../../../zig-modules/rate-limiter/build.zig");
    
    // Set library path for runtime
    println!("cargo:rustc-env=LD_LIBRARY_PATH={}", zig_lib_path.display());
}