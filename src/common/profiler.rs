use cfg_if::cfg_if;

use crate::args::ProfilerArgs;

cfg_if! {
    if #[cfg(feature = "profiler")] {
        pub fn start_puffin_server(args: &ProfilerArgs) -> Option<puffin_http::Server> {
            if let Some(endpoint) = args.puffin_endpoint {
                log::info!("Starting puffin server on {}", endpoint);
                puffin::set_scopes_on(true);
                Some(puffin_http::Server::new(&endpoint.to_string()).unwrap())
            } else {
                None
            }
        }
    } else {
        #[allow(dead_code)]
        pub fn start_puffin_server(_args: &ProfilerArgs){}
    }
}
