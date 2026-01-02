use hydroottawa_api::api_types::{HoHourlyUsage, HoProfile};
use std::fmt;
use tabled::Table;

pub struct ProfileDisplay<'a>(pub &'a HoProfile);
pub struct UsageDisplay<'a>(pub &'a HoHourlyUsage);

impl<'a> fmt::Display for ProfileDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let profile = self.0;
        writeln!(f, "\n=== Account Information ===")?;
        writeln!(f, "Account ID: {}", profile.account_information.account_id)?;
        writeln!(f, "Premise ID: {}", profile.account_information.premise_id)?;
        writeln!(
            f,
            "Pseudo Name: {}",
            profile.account_information.pseudo_name
        )?;
        writeln!(f, "\nService Address:")?;
        writeln!(
            f,
            "  {} {}{}",
            profile.account_information.service_address.street_number,
            profile.account_information.service_address.street_name,
            if profile
                .account_information
                .service_address
                .apartment
                .is_empty()
            {
                String::new()
            } else {
                format!(
                    ", Apt {}",
                    profile.account_information.service_address.apartment
                )
            }
        )?;
        writeln!(
            f,
            "  {}, {} {}",
            profile.account_information.service_address.city,
            profile.account_information.service_address.province,
            profile.account_information.service_address.postal_code
        )?;
        writeln!(f, "\nMailing Address:")?;
        writeln!(
            f,
            "  {} {}{}",
            profile.account_information.mailing_address.street_number,
            profile.account_information.mailing_address.street_name,
            if profile
                .account_information
                .mailing_address
                .apartment
                .is_empty()
            {
                String::new()
            } else {
                format!(
                    ", Apt {}",
                    profile.account_information.mailing_address.apartment
                )
            }
        )?;
        writeln!(
            f,
            "  {}, {} {}",
            profile.account_information.mailing_address.city,
            profile.account_information.mailing_address.province,
            profile.account_information.mailing_address.postal_code
        )?;
        writeln!(f, "\nContact:")?;
        if !profile.account_information.home_phone_number.is_empty() {
            writeln!(
                f,
                "  Home: {}",
                profile.account_information.home_phone_number
            )?;
        }
        if !profile.account_information.mobile_phone_number.is_empty() {
            writeln!(
                f,
                "  Mobile: {}",
                profile.account_information.mobile_phone_number
            )?;
        }
        if !profile.account_information.business_phone_number.is_empty() {
            write!(
                f,
                "  Business: {}",
                profile.account_information.business_phone_number
            )?;
            if !profile
                .account_information
                .business_phone_number_extension
                .is_empty()
            {
                write!(
                    f,
                    " x{}",
                    profile.account_information.business_phone_number_extension
                )?;
            }
            writeln!(f)?;
        }
        writeln!(f, "\n=== User Information ===")?;
        writeln!(f, "Username: {}", profile.user_information.username)?;
        writeln!(
            f,
            "Language: {}",
            profile.user_information.language_preference
        )?;
        writeln!(f, "MFA Enabled: {}", profile.user_information.mfa_enabled)?;
        writeln!(
            f,
            "Social Sign-In: {}",
            profile.user_information.social_sign_in
        )?;
        Ok(())
    }
}

impl<'a> fmt::Display for UsageDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let usage = self.0;
        writeln!(f, "\n=== Hourly Usage Summary ===")?;
        writeln!(f, "Date: {}", usage.summary.actual_date)?;
        writeln!(f, "Rate Plan: {}", usage.summary.rate_plan)?;
        writeln!(
            f,
            "Billing Period: {} to {}",
            usage.summary.billing_period_start_date, usage.summary.billing_period_end_date
        )?;
        writeln!(f, "\n--- Overall Statistics ---")?;
        writeln!(f, "Total Usage: {:.2} kWh", usage.summary.total_usage)?;
        writeln!(f, "Total Cost: ${:.2}", usage.summary.total_cost)?;
        writeln!(
            f,
            "Average Usage/Hour: {:.2} kWh",
            usage.summary.hourly_average_usage
        )?;
        writeln!(
            f,
            "Average Cost/Hour: ${:.2}",
            usage.summary.hourly_average_cost
        )?;
        writeln!(f, "Number of Hours: {}", usage.summary.number_of_hours)?;

        writeln!(f, "\n--- Usage by Rate Band ---")?;
        writeln!(
            f,
            "Off-Peak:  {:.2} kWh (${:.2})",
            usage.summary.total_off_peak_usage, usage.summary.total_off_peak_cost
        )?;
        writeln!(
            f,
            "Mid-Peak:  {:.2} kWh (${:.2})",
            usage.summary.total_mid_peak_usage, usage.summary.total_mid_peak_cost
        )?;
        writeln!(
            f,
            "On-Peak:   {:.2} kWh (${:.2})",
            usage.summary.total_on_peak_usage, usage.summary.total_on_peak_cost
        )?;
        writeln!(
            f,
            "ULO:       {:.2} kWh (${:.2})",
            usage.summary.total_ulo_usage, usage.summary.total_ulo_cost
        )?;

        writeln!(f, "\n=== Hourly Intervals ===")?;

        // Create displayable intervals for the table
        let intervals: Vec<IntervalDisplay> = usage
            .intervals
            .iter()
            .map(|i| IntervalDisplay {
                start_date_time: i.start_date_time.clone(),
                end_date_time: i.end_date_time.clone(),
                rate_band: i.rate_band.clone(),
                hourly_usage: format!("{:.2}", i.hourly_usage),
                hourly_cost: format!("{:.2}", i.hourly_cost),
            })
            .collect();

        let table = Table::new(intervals).to_string();
        write!(f, "{}", table)?;

        Ok(())
    }
}

// Helper struct for table display
#[derive(tabled::Tabled)]
struct IntervalDisplay {
    #[tabled(rename = "Start Time")]
    start_date_time: String,
    #[tabled(rename = "End Time")]
    end_date_time: String,
    #[tabled(rename = "Rate Band")]
    rate_band: String,
    #[tabled(rename = "Usage (kWh)")]
    hourly_usage: String,
    #[tabled(rename = "Cost ($)")]
    hourly_cost: String,
}
