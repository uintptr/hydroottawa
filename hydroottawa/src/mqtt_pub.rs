use anyhow::{Context, Result};
use hydroottawa_api::types::{HoHourlyUsage, HoProfile};
use log::{debug, info, warn};
use rumqttc::{Client, Connection, Event, MqttOptions, Packet, QoS};
use serde_json::json;
use std::time::Duration;

/// One Home Assistant MQTT discovery config, describing a single sensor.
struct DiscoverySensor<'a> {
    sensor_name: &'a str,
    friendly_name: &'a str,
    unit: &'a str,
    icon: &'a str,
    device_class: Option<&'a str>,
    state_class: Option<&'a str>,
}

fn publish_discovery_config(
    client: &Client,
    base_topic: &str,
    account_id: &str,
    sensor: &DiscoverySensor<'_>,
) -> Result<()> {
    // Destructure so the fields inline directly into format strings.
    let DiscoverySensor {
        sensor_name,
        friendly_name,
        unit,
        icon,
        device_class,
        state_class,
    } = *sensor;

    let config_topic = format!("{base_topic}_{sensor_name}/config");
    let state_topic = format!("hydroottawa/{account_id}/state");

    let mut config = json!({
        "name": format!("Hydro Ottawa {friendly_name}"),
        "unique_id": format!("hydroottawa_{account_id}_{sensor_name}"),
        "state_topic": state_topic,
        "value_template": format!("{{{{ value_json.{sensor_name} }}}}"),
        "unit_of_measurement": unit,
        "icon": icon,
        "device": {
            "identifiers": [format!("hydroottawa_{account_id}")],
            "name": format!("Hydro Ottawa {account_id}"),
            "manufacturer": "Hydro Ottawa",
            "model": "Energy Monitor"
        }
    });

    if let Some(dc) = device_class {
        config["device_class"] = json!(dc);
    }
    if let Some(sc) = state_class {
        config["state_class"] = json!(sc);
    }

    debug!("Publishing discovery config for {sensor_name} to {config_topic}");
    client
        .publish(&config_topic, QoS::AtLeastOnce, true, config.to_string())
        .context("publishing discovery config")?;
    Ok(())
}

/// Drive the MQTT connection until the broker has acknowledged every publish.
///
/// The blocking client only makes progress while the connection is being
/// iterated, so queued publishes are not sent until this runs.
fn drain_connection(connection: &mut Connection) -> Result<()> {
    const EXPECTED_PUBLISHES: u32 = 3; // 2 discovery configs + 1 state

    let mut publish_count = 0u32;

    for event in connection.iter() {
        match event {
            Ok(Event::Incoming(Packet::ConnAck(_))) => {
                info!("Connected to MQTT broker");
            }
            Ok(Event::Incoming(Packet::PubAck(_))) => {
                publish_count = publish_count.saturating_add(1);
                debug!("Publish acknowledged ({publish_count}/{EXPECTED_PUBLISHES})");
                if publish_count >= EXPECTED_PUBLISHES {
                    info!("All messages acknowledged by broker");
                    return Ok(());
                }
            }
            Ok(Event::Outgoing(_)) => {
                debug!("Outgoing event");
            }
            Ok(event) => {
                debug!("MQTT event: {event:?}");
            }
            Err(e) => {
                warn!("MQTT connection error: {e}");
                return Err(e.into());
            }
        }
    }

    // The iterator only ends when the request channel is closed, which
    // cannot happen while the client is still alive above us.
    anyhow::bail!("MQTT connection closed after {publish_count}/{EXPECTED_PUBLISHES} publishes")
}

/// Build the state payload from the usage summary (intervals excluded),
/// with floats rounded to 2 decimals.
fn build_state_payload(usage: &HoHourlyUsage) -> serde_json::Value {
    let round = |val: f64| (val * 100.0).round() / 100.0;

    json!({
        "accountId": usage.summary.account_id,
        "actualDate": usage.summary.actual_date,
        "ratePlan": usage.summary.rate_plan,
        "billingPeriodStartDate": usage.summary.billing_period_start_date,
        "billingPeriodEndDate": usage.summary.billing_period_end_date,
        "totalUsage": round(usage.summary.total_usage),
        "totalCost": round(usage.summary.total_cost),
        "totalOffPeakUsage": round(usage.summary.total_off_peak_usage),
        "totalOffPeakCost": round(usage.summary.total_off_peak_cost),
        "totalMidPeakUsage": round(usage.summary.total_mid_peak_usage),
        "totalMidPeakCost": round(usage.summary.total_mid_peak_cost),
        "totalOnPeakUsage": round(usage.summary.total_on_peak_usage),
        "totalOnPeakCost": round(usage.summary.total_on_peak_cost),
        "totalUloUsage": round(usage.summary.total_ulo_usage),
        "totalUloCost": round(usage.summary.total_ulo_cost),
        "numberOfHours": usage.summary.number_of_hours,
    })
}

/// Publish Home Assistant discovery configs and the usage summary state.
///
/// `server` is `host` or `host:port` (port defaults to 1883).
///
/// # Errors
///
/// Returns an error if a publish fails or the broker connection drops
/// before all messages are acknowledged.
pub fn mqtt_publish<S>(server: S, profile: &HoProfile, usage: &HoHourlyUsage) -> Result<()>
where
    S: AsRef<str>,
{
    let server = server.as_ref();
    let account_id = &profile.account_information.account_id;

    // Parse server address (format: host:port or just host, default port 1883)
    let (host, port) = if let Some((h, p)) = server.split_once(':') {
        (h, p.parse().unwrap_or(1883))
    } else {
        (server, 1883)
    };

    info!("Connecting to MQTT broker at {host}:{port}");
    let mut mqttoptions = MqttOptions::new("hydroottawa", host, port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut connection) = Client::new(mqttoptions, 10);

    // Base topic for Home Assistant MQTT discovery
    let base_topic = format!("homeassistant/sensor/hydroottawa_{account_id}");
    info!("Publishing Home Assistant discovery configs for account {account_id}");

    let sensors = [
        DiscoverySensor {
            sensor_name: "totalUsage",
            friendly_name: "Total Usage",
            unit: "kWh",
            icon: "mdi:lightning-bolt",
            device_class: Some("energy"),
            state_class: Some("total"),
        },
        DiscoverySensor {
            sensor_name: "totalCost",
            friendly_name: "Total Cost",
            unit: "CAD",
            icon: "mdi:currency-usd",
            device_class: Some("monetary"),
            state_class: Some("total"),
        },
    ];
    for sensor in &sensors {
        publish_discovery_config(&client, &base_topic, account_id, sensor)?;
    }

    // Publish summary state data
    info!("Publishing usage summary data");
    let state_topic = format!("hydroottawa/{account_id}/state");
    let state_payload = build_state_payload(usage);
    debug!("State payload: {state_payload}");

    client
        .publish(
            &state_topic,
            QoS::AtLeastOnce,
            false,
            state_payload.to_string(),
        )
        .context("publishing state")?;
    debug!("Queued state for topic: {state_topic}");

    // Everything above only queues; iterating the connection connects to
    // the broker, flushes the queue and waits for the acks.
    debug!("Waiting for all publishes to be acknowledged");
    drain_connection(&mut connection)?;

    info!("Successfully published all MQTT messages");
    Ok(())
}
