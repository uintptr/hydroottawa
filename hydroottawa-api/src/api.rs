use chrono::NaiveDate;
use reqwest::Client;
use serde::Serialize;

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

/// Client for the Hydro Ottawa `myAccount` REST API.
pub struct HoApi {
    client: Client,
    debug_responses: DebugResponses,
}

impl HoApi {
    /// Create a new API client.
    #[must_use]
    pub fn new(debug_responses: DebugResponses) -> Self {
        let client = Client::new();

        Self {
            client,
            debug_responses,
        }
    }

    /// Fetch the account profile for the authenticated user.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the response cannot
    /// be deserialized into [`HoProfile`].
    pub async fn profile(&self, auth: &HoAuth) -> Result<HoProfile> {
        let url = format!("{HO_API_URI}/profile");

        let profile_dict = self
            .client
            .get(url)
            .header("Accept", "application/json")
            .header("x-id", &auth.id_token)
            .header("x-access", &auth.access_token)
            .bearer_auth(&auth.jwt_token)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

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
    pub async fn hourly(&self, auth: &HoAuth, date: &NaiveDate) -> Result<HoHourlyUsage> {
        let url = format!("{HO_API_URI}/usage/consumption/hourly");

        let day = HourlyRequest {
            date: date.format("%Y-%m-%d").to_string(),
        };

        let hourly_dict = self
            .client
            .post(url)
            .header("Accept", "application/json")
            .header("x-id", &auth.id_token)
            .header("x-access", &auth.access_token)
            .bearer_auth(&auth.jwt_token)
            .json(&day)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        if self.debug_responses == DebugResponses::On {
            eprintln!("{hourly_dict:#?}");
        }

        let usage: HoHourlyUsage = serde_json::from_value(hourly_dict)?;

        Ok(usage)
    }
}
