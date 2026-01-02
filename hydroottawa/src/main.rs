use anyhow::Result;
use clap::Parser;
use dialoguer::Password;
use hydroottawa_api::{api::HoApi, auth::HoAuth};
use log::LevelFilter;
use rstaples::logging::StaplesLogger;
use std::env;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct UserArgs {
    /// verbose
    #[arg(short, long)]
    verbose: bool,

    /// Username
    #[arg(short, long)]
    username: String,
}

fn get_password() -> Result<String> {
    if let Ok(password) = env::var("HO_PASSWORD") {
        Ok(password)
    } else {
        let password = Password::new().with_prompt("Password").interact()?;
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

    let password = get_password()?;

    let auth = HoAuth::new(&args.username, &password).await?;
    println!("Authentication successful!");

    let api = HoApi::new(args.verbose);

    let usage = api.hourly(&auth).await?;

    dbg!(usage);

    //let profile = api.profile(&auth).await?;
    //println!("\nProfile:");
    //dbg!(profile);

    Ok(())
}
