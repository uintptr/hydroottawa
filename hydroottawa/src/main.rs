mod display;

use anyhow::Result;
use chrono::{Local, NaiveDate};
use clap::Parser;
use dialoguer::Password;
use hydroottawa::mqtt_pub::mqtt_publish;
use hydroottawa_api::{api::HoApi, auth::HoAuth};
use log::LevelFilter;
use rstaples::logging::StaplesLogger;
use std::env;

use display::{ProfileDisplay, UsageDisplay};

const FALLBACK_DATE: NaiveDate = match NaiveDate::from_ymd_opt(2025, 1, 1) {
    Some(date) => date,
    None => unreachable!(),
};

fn yesterday() -> NaiveDate {
    Local::now()
        .date_naive()
        .pred_opt()
        .unwrap_or(FALLBACK_DATE)
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct UserArgs {
    /// verbose
    #[arg(short, long)]
    verbose: bool,

    /// date
    #[arg(short, long, default_value_t = yesterday())]
    date: NaiveDate,

    /// Username
    #[arg(short, long)]
    username: String,

    /// Username
    #[arg(short, long)]
    mqtt: Option<String>,
}

fn get_password(username: &str) -> Result<String> {
    if let Ok(password) = env::var("HO_PASSWORD") {
        Ok(password)
    } else {
        let prompt = format!("Password for {username}");
        let password = Password::new().with_prompt(prompt).interact()?;
        Ok(password)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = UserArgs::parse();

    let log_level = if args.verbose {
        LevelFilter::Info
    } else {
        LevelFilter::Error
    };

    StaplesLogger::new()
        .with_colors()
        .with_log_level(log_level)
        .start();

    let password = get_password(&args.username)?;

    let auth = HoAuth::new(&args.username, &password).await?;
    println!("Authentication successful!");

    let api = HoApi::new(false);

    let profile = api.profile(&auth).await?;
    let usage = api.hourly(&auth, &args.date).await?;

    if let Some(mqtt_server) = args.mqtt {
        mqtt_publish(mqtt_server, &profile, &usage).await
    } else {
        println!("{}", ProfileDisplay(&profile));
        println!("{}", UsageDisplay(&usage));
        Ok(())
    }
}
