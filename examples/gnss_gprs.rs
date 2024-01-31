//! Example of using GNSS data, and creating dummy POST request via GRPS.

use rpi_sim868::{
    gnss::GNSSData,
    gprs::{ContentType, Request},
    SIM868,
};
use serde_json::{json, Value};
use std::{thread::sleep, time::Duration};

async fn post_request(sim: &SIM868, gnss_data: GNSSData) -> Result<String, rpi_sim868::Error> {
    let data: Value = json!({
        "alt": gnss_data.alt,
        "lat": gnss_data.lat,
        "lon": gnss_data.lon,
        "utc_datetime": format!("{}", gnss_data.utc_datetime)
    });

    let req: Request<Value> = Request {
        content_type: Some(ContentType::Json),
        data,
        userdata_header: Some(String::from("my-custom-header: key1=value1; key2=value2")),
        method: rpi_sim868::gprs::RequestMethod::POST,
        url: String::from("http://httpbin.org/post"),
    };

    Ok(sim.gprs.request(req).await??)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sim: SIM868 = SIM868::new("/dev/ttyS0", 115200, rpi_sim868::LogLevelFilter::Debug);

    // turn on hat if turned off
    if let Err(_) = sim.hat.is_on().await? {
        sim.hat.turn_on().await?
    }

    // turn on gnss module
    if !sim.gnss.is_on().await?? {
        sim.gnss.turn_on().await??;
    }

    // initialize gprs
    sim.gprs
        .init(rpi_sim868::gprs::ApnConfig {
            apn: String::from("internet"),
            user: String::new(),
            password: String::new(),
        })
        .await??;

    // wait for the network connection
    while let Ok(network_strenght) = sim.hat.network_strength().await? {
        if network_strenght > 0 {
            break;
        }
        sleep(Duration::from_secs(2));
    }

    // wait for the GNSS data and send it in the request
    loop {
        if let Ok(gnss_data) = sim.gnss.get_data().await? {
            println!("Response: {}", post_request(&sim, gnss_data).await?);
            break;
        };
        sleep(Duration::from_secs(2));
    }

    Ok(())
}
