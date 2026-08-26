use std::{fs, io::Result, path::Path};

use serde::{Deserialize, Serialize};

/// Profile response: account and user information.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoProfile {
    pub account_information: HoAccountInformation,
    pub user_information: HoUserInformation,
}

/// Account details: IDs, addresses, and phone numbers.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoAccountInformation {
    pub account_id: String,
    pub business_phone_number: String,
    pub business_phone_number_extension: String,
    pub home_phone_number: String,
    pub mailing_address: HoAddress,
    pub mobile_phone_number: String,
    pub premise_id: String,
    pub pseudo_name: String,
    pub service_address: HoAddress,
}

/// A service or mailing address.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoAddress {
    pub apartment: String,
    pub city: String,
    pub postal_code: String,
    pub province: String,
    pub street_name: String,
    pub street_number: String,
}

/// User account settings and sign-in details.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoUserInformation {
    pub language_preference: String,
    pub mfa_enabled: bool,
    pub mfa_phone_number: String,
    pub social_sign_in: bool,
    pub username: String,
}

/// One hour of usage: timestamps, rate band, kWh, and cost.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HoInterval {
    pub start_date_time: String,
    pub end_date_time: String,
    pub rate_band: String,
    pub hourly_usage: f64,
    pub hourly_cost: f64,
}

/// Daily totals and per-rate-band breakdowns.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HoSummary {
    pub account_id: String,
    pub actual_date: String,
    pub rate_plan: String,
    pub billing_period_start_date: String,
    pub billing_period_end_date: String,
    pub total_usage: f64,
    pub total_cost: f64,
    pub hourly_average_usage: f64,
    pub hourly_average_cost: f64,
    pub total_off_peak_usage: f64,
    pub total_off_peak_cost: f64,
    pub total_mid_peak_usage: f64,
    pub total_mid_peak_cost: f64,
    pub total_on_peak_usage: f64,
    pub total_on_peak_cost: f64,
    pub total_ulo_usage: f64,
    pub total_ulo_cost: f64,
    pub number_of_hours: u32,
}

/// Hourly usage response: intervals plus the daily summary.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HoHourlyUsage {
    pub intervals: Vec<HoInterval>,
    pub summary: HoSummary,
}

impl HoHourlyUsage {
    pub fn from_file<P>(file: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let sample_data = fs::read_to_string(file)?;

        let usage: HoHourlyUsage = serde_json::from_str(&sample_data)?;

        Ok(usage)
    }
}
