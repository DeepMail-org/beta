//! Build script: compile every DeepMail .proto into Rust types via tonic-build.
//!
//! Generated modules are emitted into `$OUT_DIR` and pulled in from
//! `src/lib.rs` with `tonic::include_proto!("deepmail.<service>")`.
//!
//! A FileDescriptorSet is also emitted so services can expose gRPC reflection
//! without re-parsing the .proto files at runtime.

use std::path::PathBuf;

const PROTO_FILES: &[&str] = &[
    "proto/auth.proto",
    "proto/billing.proto",
    "proto/body.proto",
    "proto/dkim.proto",
    "proto/geo.proto",
    "proto/graph.proto",
    "proto/hashdb.proto",
    "proto/header.proto",
    "proto/homograph.proto",
    "proto/intel.proto",
    "proto/ioc.proto",
    "proto/ip.proto",
    "proto/ml.proto",
    "proto/notify.proto",
    "proto/report.proto",
    "proto/sandbox.proto",
    "proto/scoring.proto",
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if which_protoc().is_none() {
        // Surface a clear, actionable error at build time rather than letting
        // tonic-build emit a less obvious failure deep in its prost invocation.
        panic!(
            "protoc not found in PATH. Install Protocol Buffers compiler:\n  \
             - Debian/Ubuntu: apt-get install -y protobuf-compiler\n  \
             - macOS (Homebrew): brew install protobuf\n  \
             - Arch Linux: pacman -S protobuf"
        );
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    let descriptor_path = out_dir.join("deepmail_descriptor.bin");

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(&descriptor_path)
        .compile(PROTO_FILES, &["proto/", "proto/google/"])?;

    println!("cargo:rerun-if-changed=build.rs");
    for proto in PROTO_FILES {
        println!("cargo:rerun-if-changed={}", proto);
    }
    Ok(())
}

fn which_protoc() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("PROTOC") {
        let pb = PathBuf::from(p);
        if pb.exists() {
            return Some(pb);
        }
    }
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join("protoc");
        if candidate.is_file() {
            return Some(candidate);
        }
        let candidate_exe = dir.join("protoc.exe");
        if candidate_exe.is_file() {
            return Some(candidate_exe);
        }
    }
    None
}
