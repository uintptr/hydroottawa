use anyhow::{Context, Result};
use chrono::{DateTime, Local, NaiveDateTime};

pub fn hydro_time_to_local<S>(date_string: S) -> Result<DateTime<Local>>
where
    S: AsRef<str>,
{
    let naive: NaiveDateTime = date_string
        .as_ref()
        .parse()
        .with_context(|| format!("Unable to parse date {}", date_string.as_ref()))?;

    let local = naive
        .and_local_timezone(Local)
        .single()
        .context("ambiguous or invalid local time")?;

    Ok(local)
}
