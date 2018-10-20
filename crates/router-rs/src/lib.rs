//! A router implementation geared towards front-end web apps

#[deny(missing_docs)]
mod router;
pub use self::router::Router;

mod route;
pub use self::route::Route;

/// Things that you'll usually need when working with frontend routing
pub mod prelude {
    pub use crate::Route;
    pub use crate::Router;
}
