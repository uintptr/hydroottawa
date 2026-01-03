use anyhow::Result;
use hydroottawa_api::types::{HoHourlyUsage, HoProfile};
use log::{debug, info, warn};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde_json::json;
use std::time::Duration;

async fn publish_discovery_config(
    client: &AsyncClient,
    base_topic: &str,
    account_id: &str,
    sensor_name: &str,
    friendly_name: &str,
    unit: &str,
    icon: &str,
    device_class: Option<&str>,
    state_class: Option<&str>,
) -> Result<()> {
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
        .await?;
    Ok(())
}

pub async fn mqtt_publish<S>(server: S, profile: &HoProfile, usage: &HoHourlyUsage) -> Result<()>
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

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // Spawn the eventloop in a background task
    let eventloop_handle = tokio::spawn(async move {
        let mut publish_count = 0;
        let expected_publishes = 3; // 2 discovery configs + 1 state

        loop {
            match eventloop.poll().await {
                Ok(Event::Incoming(Packet::ConnAck(_))) => {
                    info!("Connected to MQTT broker");
                }
                Ok(Event::Incoming(Packet::PubAck(_))) => {
                    publish_count += 1;
                    debug!(
                        "Publish acknowledged ({}/{})",
                        publish_count, expected_publishes
                    );
                    if publish_count >= expected_publishes {
                        info!("All messages acknowledged by broker");
                        break;
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
                    return Err(e);
                }
            }
        }
        Ok(())
    });

    // Give the connection a moment to establish
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Base topic for Home Assistant MQTT discovery
    let base_topic = format!("homeassistant/sensor/hydroottawa_{account_id}");
    info!("Publishing Home Assistant discovery configs for account {account_id}");

    // Publish discovery configs for various sensors
    publish_discovery_config(
        &client,
        &base_topic,
        account_id,
        "totalUsage",
        "Total Usage",
        "kWh",
        "mdi:lightning-bolt",
        Some("energy"),
        Some("total"),
    )
    .await?;
    publish_discovery_config(
        &client,
        &base_topic,
        account_id,
        "totalCost",
        "Total Cost",
        "CAD",
        "mdi:currency-usd",
        Some("monetary"),
        Some("total"),
    )
    .await?;

    // Publish summary state data
    info!("Publishing usage summary data");
    let state_topic = format!("hydroottawa/{account_id}/state");

    // Helper function to round to 2 decimals
    let round = |val: f64| (val * 100.0).round() / 100.0;

    // Create state payload with flattened summary fields (no intervals) and rounded values
    let state_payload = json!({
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
    });
    debug!("State payload: {state_payload}");

    client
        .publish(
            &state_topic,
            QoS::AtLeastOnce,
            false,
            state_payload.to_string(),
        )
        .await?;
    debug!("Published state to topic: {state_topic}");

    // Wait for the eventloop task to finish (all publishes acknowledged)
    debug!("Waiting for all publishes to be acknowledged");
    match eventloop_handle.await {
        Ok(Ok(())) => {
            info!("Successfully published all MQTT messages");
            Ok(())
        }
        Ok(Err(e)) => {
            warn!("MQTT eventloop error: {e}");
            Err(e.into())
        }
        Err(e) => {
            warn!("Failed to join eventloop task: {e}");
            Err(e.into())
        }
    }
}
