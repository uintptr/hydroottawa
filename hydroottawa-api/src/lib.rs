//! Client library for the Hydro Ottawa `myAccount` API.

pub mod api;
pub mod auth;
pub mod error;
pub mod types;

pub(crate) const HO_API_URI: &str = "https://api-myaccount.hydroottawa.com";
