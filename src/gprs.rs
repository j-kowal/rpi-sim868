//! GPRS module
//!
//! See [`GPRS`] to discover available methods.
//!
//! The [`GPRS::request`] method performs HTTP requests. The [`Request::data`] must implement the [`serde::Serialize`] trait.
//!
//! #### Request's data examples
//! ```
//! #[derive(serde::Serialize)]
//! struct Coordinates {
//!     lat: f32,
//!     lon: f32,
//! }
//!
//! // OR you can also use serde_json crate.
//!
//! let data = r#"{
//!    "name": "John Doe",
//!    "age": 43,
//!    "phones": [
//!        "+44 1234567",
//!        "+44 2345678"
//!    ]
//! }"#;
//!
//! // serde_json::Value also implements serde::Serialize.
//! let v: Value = serde_json::from_str(data).unwrap();
//! ```
//!
//! ⚠️ Unfortunately, the SIM868 doesn't support HTTPS requests, so please use HTTP.
//!
//! ⚠️ Prior to use for making requests, it is crucial to execute the [`GPRS::init`]
//! method with your [Access Point Name (APN) configuration](`ApnConfig`),
//! ensuring the GPRS connection can be made.

use crate::{
    error::Error,
    error_check, generic_resolver, http,
    serial_port::{spawn_task, SerialPort, TaskPriority},
    Module, ResolverReturn, TaskJoinHandle, GPRS_CONN_STATUS_REGEX, PARSING_ERROR,
};
use std::{sync::Arc, time::Duration};
use uuid::Uuid;

fn conn_status(serial_port: &Arc<SerialPort>, task_id: &Uuid) -> ResolverReturn<u8> {
    fn resolver(result: String) -> ResolverReturn<u8> {
        if error_check(&result) {
            return Err(Error::GprsNoConnection);
        }
        if let Some(captured) = GPRS_CONN_STATUS_REGEX.captures(&result) {
            let res: &Vec<&str> = &captured["data"].split(",").collect();
            Ok(res[1].parse::<u8>().expect(PARSING_ERROR))
        } else {
            return Err(Error::NotResolved);
        }
    }

    serial_port.process(task_id, "AT+SAPBR=2,1\n".to_string(), resolver, None)
}

fn conn_open(serial_port: &Arc<SerialPort>, task_id: &Uuid) -> ResolverReturn<()> {
    fn resolver(result: String) -> ResolverReturn<()> {
        return generic_resolver(&result, Error::GprsConnectionOpenFailed);
    }

    serial_port.process(
        task_id,
        "AT+SAPBR=1,1\n".to_string(),
        resolver,
        Some(Duration::from_secs(20)),
    )
}

fn conn_close(serial_port: &Arc<SerialPort>, task_id: &Uuid, _: ()) -> ResolverReturn<()> {
    fn resolver(result: String) -> ResolverReturn<()> {
        return generic_resolver(&result, Error::GprsConnectionCloseFailed);
    }

    serial_port.process(
        task_id,
        "AT+CGATT=0\n".to_string(),
        resolver,
        Some(Duration::from_secs(10)),
    )
}

fn init(
    serial_port: &Arc<SerialPort>,
    task_id: &Uuid,
    apn_config: ApnConfig,
) -> ResolverReturn<()> {
    fn resolver(result: String) -> ResolverReturn<()> {
        generic_resolver(&result, Error::GprsApnConfigSetFailed)
    }

    let commands: [String; 4] = [
        "AT+SAPBR=3,1,Contype,GPRS\n".to_string(),
        format!("AT+SAPBR=3,1,APN,\"{}\"\n", apn_config.apn),
        format!("AT+SAPBR=3,1,USER,\"{}\"\n", apn_config.user),
        format!("AT+SAPBR=3,1,PWD,\"{}\"\n", apn_config.password),
    ];

    for command in commands {
        serial_port.process(task_id, command, resolver, None)?
    }

    Ok(())
}

fn request<T>(
    serial_port: &Arc<SerialPort>,
    task_id: &Uuid,
    req: Request<T>,
) -> ResolverReturn<String>
where
    T: serde::Serialize,
{
    // terminate - just in case if previous http was initiated and wasn't terminated afterwards
    let _ = http::terminate(serial_port, task_id);
    let status: u8 = conn_status(serial_port, task_id)?;
    if status == 3 {
        conn_open(serial_port, task_id)?;
    }
    http::init(serial_port, task_id, &req)?;
    if matches!(req.method, RequestMethod::POST) {
        http::data(serial_port, task_id, &req)?;
    }
    http::action(serial_port, task_id, req.method)?;
    let read: String = http::read(serial_port, task_id)?;
    http::terminate(serial_port, task_id)?;
    Ok(read)
}

fn request_wrapper<T>(
    serial_port: &Arc<SerialPort>,
    task_id: &Uuid,
    req: Request<T>,
) -> ResolverReturn<String>
where
    T: serde::Serialize,
{
    let result: Result<String, Error> = request(serial_port, task_id, req);
    // always close the connection afterwards
    conn_close(serial_port, task_id, ())?;
    result
}

pub struct ApnConfig {
    pub apn: String,
    pub user: String,
    pub password: String,
}

pub struct GPRS {
    serial_port: Arc<SerialPort>,
}

impl Module for GPRS {
    fn new(serial_port: Arc<crate::serial_port::SerialPort>) -> Self {
        GPRS { serial_port }
    }
}

#[derive(PartialEq, Debug)]
pub enum RequestMethod {
    GET,
    POST,
    HEAD,
}

#[derive(Clone, Copy)]
pub enum ContentType {
    FormUrlencoded,
    Json,
}

pub struct Request<T>
where
    T: serde::Serialize,
{
    pub content_type: Option<ContentType>,
    pub data: T,
    pub headers: Option<String>,
    pub method: RequestMethod,
    pub url: String,
}

impl GPRS {
    /// Creates request GET, POST, or HEAD. Because of SIM868 limitations, HTTPS requests are not supported.
    pub fn request<T>(&self, req: Request<T>) -> TaskJoinHandle<String>
    where
        T: serde::Serialize + Send + 'static,
    {
        spawn_task(
            self.serial_port.clone(),
            TaskPriority::NORMAL,
            request_wrapper,
            Some(format!(
                "Creating {:?} request to {}...",
                req.method, req.url
            )),
            req,
        )
    }

    /// The APN should be initialised before using GPRS.
    pub fn init(&self, apn_config: ApnConfig) -> TaskJoinHandle<()> {
        spawn_task(
            self.serial_port.clone(),
            TaskPriority::NORMAL,
            init,
            Some("Setting APN config...".to_string()),
            apn_config,
        )
    }

    /// Closes GPRS connection
    pub fn close_connection(&self) -> TaskJoinHandle<()> {
        spawn_task(
            self.serial_port.clone(),
            TaskPriority::NORMAL,
            conn_close,
            Some("Setting APN config...".to_string()),
            (),
        )
    }
}
