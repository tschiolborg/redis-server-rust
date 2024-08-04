use crate::data::SharedData;
use std::time::Duration;
use tokio::time;

pub async fn delete_expired(data: SharedData) {
    let mut interval = time::interval(Duration::from_secs(6));

    loop {
        interval.tick().await;

        println!("(INFO) Checking for expired keys");

        let mut data = data.write().await;

        data.expire_keys();
    }
}
