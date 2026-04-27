fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos = &[
        "proto/auth.proto",
        "proto/billing.proto",
        "proto/dkim.proto",
        "proto/geo.proto",
        "proto/graph.proto",
        "proto/hashdb.proto",
        "proto/header.proto",
        "proto/homograph.proto",
        "proto/intel.proto",
        "proto/ioc.proto",
        "proto/ml.proto",
        "proto/notify.proto",
        "proto/report.proto",
        "proto/sandbox.proto",
        "proto/scoring.proto",
    ];

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, &["proto/"])?;

    for proto in protos {
        println!("cargo:rerun-if-changed={proto}");
    }

    Ok(())
}
