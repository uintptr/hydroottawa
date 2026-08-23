use hydroottawa_api::types::{HoAddress, HoHourlyUsage, HoProfile};
use std::fmt;
use tabled::Table;

/// Display adapter for printing a [`HoProfile`] as text.
pub struct ProfileDisplay<'a>(pub &'a HoProfile);

/// Display adapter for printing a [`HoHourlyUsage`] summary and table.
pub struct UsageDisplay<'a>(pub &'a HoHourlyUsage);

fn write_address(f: &mut fmt::Formatter<'_>, address: &HoAddress) -> fmt::Result {
    let apartment = if address.apartment.is_empty() {
        String::new()
    } else {
        format!(", Apt {}", address.apartment)
    };
    writeln!(
        f,
        "  {} {}{apartment}",
        address.street_number, address.street_name
    )?;
    writeln!(
        f,
        "  {}, {} {}",
        address.city, address.province, address.postal_code
    )
}

impl fmt::Display for ProfileDisplay<'_> {
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
        write_address(f, &profile.account_information.service_address)?;
        writeln!(f, "\nMailing Address:")?;
        write_address(f, &profile.account_information.mailing_address)?;
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
            if !profile.account_information.business_phone_number_extension.is_empty() {
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

impl fmt::Display for UsageDisplay<'_> {
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

        // Create displayable intervals for the table (borrows the strings,
        // so no per-row String allocations)
        let intervals: Vec<IntervalDisplay<'_>> = usage
            .intervals
            .iter()
            .map(|i| IntervalDisplay {
                start_date_time: &i.start_date_time,
                end_date_time: &i.end_date_time,
                rate_band: &i.rate_band,
                hourly_usage: format!("{:.2}", i.hourly_usage),
                hourly_cost: format!("{:.2}", i.hourly_cost),
            })
            .collect();

        let table = Table::new(intervals).to_string();
        write!(f, "{table}")?;

        Ok(())
    }
}

// Helper struct for table display
#[derive(tabled::Tabled)]
struct IntervalDisplay<'a> {
    #[tabled(rename = "Start Time")]
    start_date_time: &'a str,
    #[tabled(rename = "End Time")]
    end_date_time: &'a str,
    #[tabled(rename = "Rate Band")]
    rate_band: &'a str,
    #[tabled(rename = "Usage (kWh)")]
    hourly_usage: String,
    #[tabled(rename = "Cost ($)")]
    hourly_cost: String,
}
