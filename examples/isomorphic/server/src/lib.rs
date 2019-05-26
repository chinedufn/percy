#![feature(proc_macro_hygiene, decl_macro)]

#[cfg(feature = "with-rocket")]
#[macro_use]
#[cfg(feature = "with-rocket")]
extern crate rocket;

#[cfg(feature = "with-actix")]
pub mod actix_server;
#[cfg(feature = "with-rocket")]
pub mod rocket_server;