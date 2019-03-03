//! A router implementation geared towards front-end web apps

#![feature(proc_macro_hygiene)]
#[deny(missing_docs)]
mod route;
mod router;
mod provided;

use route::Route;

/// Things that you'll usually need when working with frontend routing
pub mod prelude {
    pub use crate::route::Route;
    pub use crate::route::RouteParam;
    pub use crate::router::RouteHandler;
    pub use crate::router::Router;
    pub use crate::provided::Provided;
    pub use crate::router::ProvidedMap;
}
