//! A router implementation geared towards front-end web apps

#![feature(proc_macro_hygiene)]
mod provided;
#[deny(missing_docs)]
mod route;
mod router;

use route::Route;

/// Things that you'll usually need when working with frontend routing
pub mod prelude {
    pub use crate::provided::Provided;
    pub use crate::route::ParseRouteParam;
    pub use crate::route::Route;
    pub use crate::route::RouteParam;
    pub use crate::router::ProvidedMap;
    pub use crate::router::RouteHandler;
    pub use crate::router::Router;
    pub use router_rs_macro::create_routes;
    pub use router_rs_macro::route;
}
