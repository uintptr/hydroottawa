use anyhow::Result;
use hydroottawa_api::{
    api::{DebugResponses, HoApi},
    auth::HoAuth,
};
use log::info;

pub async fn spoon_feed<U, P, M>(user: U, password: P, _mqtt_server: M) -> Result<()>
where
    U: AsRef<str>,
    P: AsRef<str>,
    M: AsRef<str>,
{
    let _api = HoApi::new(DebugResponses::Off);

    let _auth = HoAuth::new(user, password).await?;

    info!("authenticated");

    Ok(())
}
