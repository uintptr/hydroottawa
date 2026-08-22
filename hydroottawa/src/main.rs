use anyhow::{Context, Result};
use chrono::{Local, NaiveDate};
use clap::Parser;
use dialoguer::Password;
use hydroottawa::display::{ProfileDisplay, UsageDisplay};
use hydroottawa::mqtt_pub::mqtt_publish;
use hydroottawa::spoon_feed::spoon_feed;
use hydroottawa_api::{
    api::{DebugResponses, HoApi},
    auth::HoAuth,
};
use log::{LevelFilter, info};
use rstaples::logging::StaplesLogger;
use std::{env, process::Command};
use which::which;

const FALLBACK_DATE: NaiveDate = match NaiveDate::from_ymd_opt(2025, 1, 1) {
    Some(date) => date,
    None => unreachable!(),
};

fn yesterday() -> NaiveDate {
    Local::now().date_naive().pred_opt().unwrap_or(FALLBACK_DATE)
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct UserArgs {
    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Date to fetch usage for (defaults to yesterday)
    #[arg(short, long, default_value_t = yesterday())]
    date: NaiveDate,

    /// Hydro Ottawa username (email)
    #[arg(short, long, env = "HO_USERNAME")]
    username: String,

    /// Command to run to retrieve the password
    #[arg(short, long, env = "HO_PASSWORD_COMMAND")]
    password_command: Option<String>,

    /// MQTT broker (host or host:port) to publish to instead of printing
    #[arg(short, long)]
    mqtt: Option<String>,

    /// Output as JSON instead of the default ASCII table
    #[arg(long)]
    json: bool,

    /// Loop, spoon feed the hourly usage
    #[arg(long)]
    spoon_feed: bool,
}

fn get_password_from_command(command: &str) -> Result<String> {
    let args =
        shell_words::split(command).with_context(|| format!("unable to tokeniz {command}"))?;

    let (command, args) = args.split_first().context("couple split command")?;

    let command = which(command).with_context(|| format!("{command} was not found in PATH"))?;

    let output = Command::new(command).args(args).output()?;

    String::from_utf8(output.stdout).context("output not utf-8")
}

fn get_password(username: &str, password_command: Option<&String>) -> Result<String> {
    if let Ok(password) = env::var("HO_PASSWORD") {
        Ok(password)
    } else if let Some(command) = password_command {
        get_password_from_command(command)
    } else if let Ok(command) = env::var("HO_PASSWORD_COMMAND") {
        get_password_from_command(&command)
    } else {
        let prompt = format!("Password for {username}");
        let password = Password::new().with_prompt(prompt).interact()?;
        Ok(password)
    }
}

fn main() -> Result<()> {
    let args = UserArgs::parse();

    let log_level = if args.verbose {
        LevelFilter::Info
    } else {
        LevelFilter::Error
    };

    StaplesLogger::new().with_colors().with_log_level(log_level).start();

    let password = get_password(&args.username, args.password_command.as_ref())?;

    if args.spoon_feed {
        let mqtt_server = args.mqtt.context("mqtt server is missing")?;

        spoon_feed(args.username, password, mqtt_server)
    } else {
        let auth = HoAuth::new(&args.username, &password)?;
        info!("Authentication successful!");

        let api = HoApi::new(DebugResponses::Off);

        let profile = api.profile(&auth)?;
        let usage = api.hourly(&auth, &args.date)?;

        if let Some(mqtt_server) = args.mqtt {
            mqtt_publish(mqtt_server, &profile, &usage)
        } else if args.json {
            let json_str = serde_json::to_string_pretty(&usage)?;
            println!("{json_str}");
            Ok(())
        } else {
            println!("{}", ProfileDisplay(&profile));
            println!("{}", UsageDisplay(&usage));
            Ok(())
        }
    }
}
