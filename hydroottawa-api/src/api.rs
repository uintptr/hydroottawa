use chrono::NaiveDate;
use serde::Serialize;
use ureq::Agent;

use crate::{
    HO_API_URI,
    auth::HoAuth,
    error::Result,
    types::{HoHourlyUsage, HoProfile},
};

#[derive(Serialize)]
struct HourlyRequest {
    date: String,
}

/// Controls whether raw API responses are dumped to stderr for debugging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugResponses {
    /// Do not dump responses.
    Off,
    /// Pretty-print the raw JSON response to stderr before deserializing.
    On,
}

/// Format a JWT as an `Authorization` header value.
///
/// ureq has no `bearer_auth` shorthand, so build the value by hand.
fn bearer(jwt: &str) -> String {
    format!("Bearer {jwt}")
}

/// Client for the Hydro Ottawa `myAccount` REST API.
pub struct HoApi {
    agent: Agent,
    debug_responses: DebugResponses,
}

impl HoApi {
    /// Create a new API client.
    #[must_use]
    pub fn new(debug_responses: DebugResponses) -> Self {
        let agent = Agent::new_with_defaults();

        Self {
            agent,
            debug_responses,
        }
    }

    /// Fetch the account profile for the authenticated user.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the response cannot
    /// be deserialized into [`HoProfile`].
    pub fn profile(&self, auth: &HoAuth) -> Result<HoProfile> {
        let url = format!("{HO_API_URI}/profile");

        let profile_dict = self
            .agent
            .get(url)
            .header("Accept", "application/json")
            .header("x-id", &auth.id_token)
            .header("x-access", &auth.access_token)
            .header("Authorization", bearer(&auth.jwt_token))
            .call()?
            .body_mut()
            .read_json::<serde_json::Value>()?;

        if self.debug_responses == DebugResponses::On {
            eprintln!("{profile_dict:#?}");
        }

        let profile: HoProfile = serde_json::from_value(profile_dict)?;

        Ok(profile)
    }

    /// Fetch hourly usage intervals and the summary for `date`.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the response cannot
    /// be deserialized into [`HoHourlyUsage`].
    pub fn hourly(&self, auth: &HoAuth, date: &NaiveDate) -> Result<HoHourlyUsage> {
        let url = format!("{HO_API_URI}/usage/consumption/hourly");

        let day = HourlyRequest {
            date: date.format("%Y-%m-%d").to_string(),
        };

        let hourly_dict = self
            .agent
            .post(url)
            .header("Accept", "application/json")
            .header("x-id", &auth.id_token)
            .header("x-access", &auth.access_token)
            .header("Authorization", bearer(&auth.jwt_token))
            .send_json(&day)?
            .body_mut()
            .read_json::<serde_json::Value>()?;

        if self.debug_responses == DebugResponses::On {
            eprintln!("{hourly_dict:#?}");
        }

        let usage: HoHourlyUsage = serde_json::from_value(hourly_dict)?;

        Ok(usage)
    }
}
