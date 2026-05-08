//! DeepMail shared library.
//!
//! Provides: proto-generated types, database helpers, NATS helpers,
//! shared error types, and configuration utilities.
//!
//! Every DeepMail service depends on this crate.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod config;
pub mod db;
pub mod error;
pub mod nats;

/// Proto-generated gRPC types for all DeepMail services.
pub mod proto {
    /// Authentication service types.
    #[allow(missing_docs, clippy::all)]
    pub mod auth {
        tonic::include_proto!("deepmail.auth");
    }

    /// Billing metering types.
    #[allow(missing_docs, clippy::all)]
    pub mod billing {
        tonic::include_proto!("deepmail.billing");
    }

    /// DKIM replay analysis types.
    #[allow(missing_docs, clippy::all)]
    pub mod dkim {
        tonic::include_proto!("deepmail.dkim");
    }

    /// Geolocation intelligence types.
    #[allow(missing_docs, clippy::all)]
    pub mod geo {
        tonic::include_proto!("deepmail.geo");
    }

    /// Entity relationship graph types.
    #[allow(missing_docs, clippy::all)]
    pub mod graph {
        tonic::include_proto!("deepmail.graph");
    }

    /// File hash registry types.
    #[allow(missing_docs, clippy::all)]
    pub mod hashdb {
        tonic::include_proto!("deepmail.hashdb");
    }

    /// Email header forensics types.
    #[allow(missing_docs, clippy::all)]
    pub mod header {
        tonic::include_proto!("deepmail.header");
    }

    /// IDN homograph detection types.
    #[allow(missing_docs, clippy::all)]
    pub mod homograph {
        tonic::include_proto!("deepmail.homograph");
    }

    /// External threat intelligence types.
    #[allow(missing_docs, clippy::all)]
    pub mod intel {
        tonic::include_proto!("deepmail.intel");
    }

    /// IOC extraction types.
    #[allow(missing_docs, clippy::all)]
    pub mod ioc {
        tonic::include_proto!("deepmail.ioc");
    }

    /// IP reputation intelligence types.
    #[allow(missing_docs, clippy::all)]
    pub mod ip {
        tonic::include_proto!("deepmail.ip");
    }

    /// AI/ML inference types.
    #[allow(missing_docs, clippy::all)]
    pub mod ml {
        tonic::include_proto!("deepmail.ml");
    }

    /// Alert and notification types.
    #[allow(missing_docs, clippy::all)]
    pub mod notify {
        tonic::include_proto!("deepmail.notify");
    }

    /// Report generation types.
    #[allow(missing_docs, clippy::all)]
    pub mod report {
        tonic::include_proto!("deepmail.report");
    }

    /// Sandbox analysis types.
    #[allow(missing_docs, clippy::all)]
    pub mod sandbox {
        tonic::include_proto!("deepmail.sandbox");
    }

    /// Threat scoring types.
    #[allow(missing_docs, clippy::all)]
    pub mod scoring {
        tonic::include_proto!("deepmail.scoring");
    }

    /// Raw FileDescriptorSet bytes for all DeepMail services. Useful for
    /// wiring up gRPC reflection (`tonic-reflection`) without re-parsing
    /// the .proto files at runtime.
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        include_bytes!(concat!(env!("OUT_DIR"), "/deepmail_descriptor.bin"));
}
