//! This module contains the definitions for each binary's CLI arguments and config file structure for the sake of readability.

use cfg_if::cfg_if;

pub mod protomask;
pub mod protomask_clat;


// Used to trick the build process into including a CLI argument based on a feature flag
cfg_if! {
    if #[cfg(feature = "profiler")] {
        #[derive(Debug, clap::Args)]
        pub struct ProfilerArgs {
            /// Expose the puffin HTTP server on this endpoint
            #[clap(long)]
            pub puffin_endpoint: Option<std::net::SocketAddr>,
        }
    } else {
        #[derive(Debug, clap::Args)]
        pub struct ProfilerArgs;
    }
}
