pub mod config;
pub mod db;
pub mod error;
pub mod nats;

/// Auto-generated protobuf/gRPC code.
/// Each sub-module corresponds to a `.proto` file in `proto/`.
pub mod proto {
    pub mod auth {
        tonic::include_proto!("deepmail.auth");
    }
    pub mod billing {
        tonic::include_proto!("deepmail.billing");
    }
    pub mod dkim {
        tonic::include_proto!("deepmail.dkim");
    }
    pub mod geo {
        tonic::include_proto!("deepmail.geo");
    }
    pub mod graph {
        tonic::include_proto!("deepmail.graph");
    }
    pub mod hashdb {
        tonic::include_proto!("deepmail.hashdb");
    }
    pub mod header {
        tonic::include_proto!("deepmail.header");
    }
    pub mod homograph {
        tonic::include_proto!("deepmail.homograph");
    }
    pub mod intel {
        tonic::include_proto!("deepmail.intel");
    }
    pub mod ioc {
        tonic::include_proto!("deepmail.ioc");
    }
    pub mod ml {
        tonic::include_proto!("deepmail.ml");
    }
    pub mod notify {
        tonic::include_proto!("deepmail.notify");
    }
    pub mod report {
        tonic::include_proto!("deepmail.report");
    }
    pub mod sandbox {
        tonic::include_proto!("deepmail.sandbox");
    }
    pub mod scoring {
        tonic::include_proto!("deepmail.scoring");
    }
}
