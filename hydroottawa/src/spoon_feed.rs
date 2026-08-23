use std::{thread::sleep, time::Duration};

use anyhow::{Result, bail};
use hydroottawa_api::{
    api::{DebugResponses, HoApi},
    auth::HoAuth,
};
use log::info;

const AUTH_ATTEMPTS: usize = 5;

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

pub fn spoon_feed<U, P, M>(user: U, password: P, _mqtt_server: M) -> Result<()>
where
    U: AsRef<str>,
    P: AsRef<str>,
    M: AsRef<str>,
{
    let _api = HoApi::new(DebugResponses::Off);

    loop {
        let _auth = auth_loop(&user, &password)?;
        info!("authenticated");
    }
}
