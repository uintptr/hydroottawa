use std::{thread::sleep, time::Duration};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Datelike, Days, Local, NaiveDate, TimeZone};
use hydroottawa_api::{
    api::{DebugResponses, HoApi},
    auth::HoAuth,
    types::HoHourlyUsage,
};
use log::info;

const AUTH_ATTEMPTS: usize = 5;
const SLEEP_UNTIL_HOUR: u32 = 3;
const SLEEP_UNTIL_MINUTE: u32 = 3;
const SLEEP_UNTIL_SECONDS: u32 = 3;

fn yesterday() -> Result<NaiveDate> {
    Local::now().date_naive().pred_opt().context("Unable to get yesterday's date")
}

fn sleep_time(hour: u32, minutes: u32, seconds: u32) -> Result<Duration> {
    let now = Local::now();

    let mut target: DateTime<Local> = Local
        .with_ymd_and_hms(now.year(), now.month(), now.day(), hour, minutes, seconds)
        .earliest()
        .context("3am does not exist today in local time")?;

    if target <= now {
        target = target
            .checked_add_days(Days::new(1))
            .context("no valid 3am tomorrow in local time")?;
    }

    (target - now).to_std().context("target is in the past")
}

fn auth_loop<U, P>(user: U, password: P) -> Result<HoAuth>
where
    U: AsRef<str>,
    P: AsRef<str>,
{
    for _ in 0..AUTH_ATTEMPTS {
        if let Ok(auth) = HoAuth::new(&user, &password) {
            return Ok(auth);
        }
        sleep(Duration::from_secs(10));
    }

    bail!("Unable to authenticate");
}

pub fn spoon_feed_usage(_usage: HoHourlyUsage) -> Result<()> {
    Ok(())
}

pub fn spoon_feed<U, P, M>(user: U, password: P, _mqtt_server: M) -> Result<()>
where
    U: AsRef<str>,
    P: AsRef<str>,
    M: AsRef<str>,
{
    let api = HoApi::new(DebugResponses::Off);

    loop {
        let auth = auth_loop(&user, &password)?;
        let yesterday = yesterday()?;
        let usage = api.hourly(&auth, &yesterday)?;

        spoon_feed_usage(usage)?;

        let sleep_time = sleep_time(SLEEP_UNTIL_HOUR, SLEEP_UNTIL_MINUTE, SLEEP_UNTIL_SECONDS)?;
        info!("sleeping for {} seconds", sleep_time.as_secs());
        sleep(sleep_time);
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use anyhow::Result;
    use hydroottawa_api::types::HoHourlyUsage;
    use log::info;

    use crate::util::hydro_time_to_local;

    #[test]
    fn parse_usage() -> Result<()> {
        env_logger::init();

        let root = env!("CARGO_MANIFEST_DIR");
        let sample = PathBuf::from(root).join("samples").join("usage.json");

        let sample_data = fs::read_to_string(&sample)?;

        let usage: HoHourlyUsage = serde_json::from_str(&sample_data)?;

        let mut total = 0.0;

        for int in usage.intervals {
            hydro_time_to_local(&int.start_date_time)?;
            total += int.hourly_cost;
        }

        info!("total: {total}");

        Ok(())
    }
}
