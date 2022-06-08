//! All of the sunbeam CSS used in this crate.

include!(concat!(env!("OUT_DIR"), "/all_sunbeam_css.rs"));

#[allow(missing_docs)]
pub const SUNBEAM_CONFIG_YML: &'static str = include_str!("../Sunbeam.yml");
