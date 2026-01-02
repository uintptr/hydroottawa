use anyhow::{Context, Result};
use chrono::NaiveDate;
use reqwest::Client;
use serde::Serialize;

use crate::{
    api_types::{HoHourlyUsage, HoProfile},
    auth::HoAuth,
};

#[derive(Serialize)]
struct HourlyRequest {
    date: String,
}

pub struct HoApi {
    client: Client,
    debug_responses: bool,
}

const HO_API_URI: &str = "https://api-myaccount.hydroottawa.com";

impl HoApi {
    #[must_use]
    pub fn new(debug_responses: bool) -> Self {
        let client = Client::new();

        Self {
            client,
            debug_responses,
        }
    }

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
            .await
            .context("Failed to fetch profile")?
            .json::<serde_json::Value>()
            .await
            .context("Failed to parse profile response")?;

        if self.debug_responses {
            dbg!(&profile_dict);
        }

        let profile: HoProfile = serde_json::from_value(profile_dict)?;

        Ok(profile)
    }

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
            .await
            .context("Failed to fetch profile")?
            .json::<serde_json::Value>()
            .await
            .context("Failed to parse profile response")?;

        if self.debug_responses {
            dbg!(&hourly_dict);
        }

        let usage: HoHourlyUsage = serde_json::from_value(hourly_dict)?;

        Ok(usage)
    }
}
