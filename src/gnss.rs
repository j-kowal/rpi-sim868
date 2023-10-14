//! GNSS module
//!
//! See [`GNSS`] to discover available methods.
//!
//! ⚠️ Please remember to turn on the GPS module by [`GNSS::turn_on`] before attempting to check for localization.

use crate::{
    error::Error,
    generic_resolver,
    serial_port::{spawn_task, SerialPort, TaskPriority},
    Module, ResolverReturn, TaskJoinHandle, GNSS_DATA_REGEX, GNSS_POWER_REGEX, PARSING_ERROR,
};
use chrono::{TimeZone, Utc};
use std::sync::Arc;
use uuid::Uuid;

/// Type returned from [`GNSS::get_data`] method.
#[derive(Debug)]
pub struct GNSSData {
    pub lat: f32,
    pub lon: f32,
    /// Meters above MSL
    pub alt: f32,
    /// km/h
    pub ground_speed: f32,
    /// degrees
    pub ground_course: f32,
    pub sats_in_view: u8,
    pub sats_in_use: u8,
    pub utc_datetime: chrono::DateTime<Utc>,
}

fn get_data(serial_port: &Arc<SerialPort>, task_id: &Uuid, _: ()) -> ResolverReturn<GNSSData> {
    fn resolver(result: String) -> ResolverReturn<GNSSData> {
        let Some(captured) = GNSS_DATA_REGEX.captures(&result) else {
            return Err(Error::NotResolved);
        };

        let data: &Vec<&str> = &captured["data"].split(",").collect();

        if data[0].parse::<u8>().expect(PARSING_ERROR) == 0 {
            return Err(Error::GnssModuleOff);
        }
        if data[1].parse::<u8>().expect(PARSING_ERROR) == 0 {
            return Err(Error::GnssNotFixed);
        }

        let year: &str = &data[2][..=3];
        let month: &str = &data[2][4..=5];
        let day: &str = &data[2][6..=7];
        let hour: &str = &data[2][8..=9];
        let minutes: &str = &data[2][10..=11];
        let seconds: &str = &data[2][12..=13];

        let utc_datetime: chrono::DateTime<Utc> = Utc
            .with_ymd_and_hms(
                year.parse().expect(PARSING_ERROR),
                month.parse().expect(PARSING_ERROR),
                day.parse().expect(PARSING_ERROR),
                hour.parse().expect(PARSING_ERROR),
                minutes.parse().expect(PARSING_ERROR),
                seconds.parse().expect(PARSING_ERROR),
            )
            .unwrap();

        Ok(GNSSData {
            utc_datetime,
            lat: data[3].parse().expect(PARSING_ERROR),
            lon: data[4].parse().expect(PARSING_ERROR),
            alt: data[5].parse().expect(PARSING_ERROR),
            ground_speed: data[6].parse().expect(PARSING_ERROR),
            ground_course: data[7].parse().expect(PARSING_ERROR),
            sats_in_view: data[14].parse().expect(PARSING_ERROR),
            sats_in_use: data[15].parse().expect(PARSING_ERROR),
        })
    }

    serial_port.process(task_id, "AT+CGNSINF\n".to_string(), resolver, None)
}

fn is_on(serial_port: &Arc<SerialPort>, task_id: &Uuid, _: ()) -> ResolverReturn<bool> {
    fn resolver(result: String) -> ResolverReturn<bool> {
        match GNSS_POWER_REGEX.captures(&result) {
            Some(captured) => {
                let status: u8 = captured["number"].parse().expect(PARSING_ERROR);
                Ok(if status == 1 { true } else { false })
            }
            None => Err(Error::NotResolved),
        }
    }

    serial_port.process(task_id, "AT+CGNSPWR?\n".to_string(), resolver, None)
}

fn turn_on(serial_port: &Arc<SerialPort>, task_id: &Uuid, _: ()) -> ResolverReturn<()> {
    fn resolver(result: String) -> ResolverReturn<()> {
        generic_resolver(&result, Error::GnssProblem)
    }
    serial_port.process(task_id, "AT+CGNSPWR=1\n".to_string(), resolver, None)
}

fn turn_off(serial_port: &Arc<SerialPort>, task_id: &Uuid, _: ()) -> ResolverReturn<()> {
    fn resolver(result: String) -> ResolverReturn<()> {
        generic_resolver(&result, Error::GnssProblem)
    }
    serial_port.process(task_id, "AT+CGNSPWR=0\n".to_string(), resolver, None)
}

/// GNSS Module
pub struct GNSS {
    serial_port: Arc<SerialPort>,
}

impl Module for GNSS {
    fn new(serial_port: Arc<SerialPort>) -> Self {
        GNSS { serial_port }
    }
}

impl GNSS {
    /// Checks if GPRS module is switched on.
    pub fn is_on(&self) -> TaskJoinHandle<bool> {
        spawn_task(
            self.serial_port.clone(),
            TaskPriority::NORMAL,
            is_on,
            Some("Checking GNSS module status...".to_string()),
            (),
        )
    }

    /// Turns GNSS module on.
    pub fn turn_on(&self) -> TaskJoinHandle<()> {
        spawn_task(
            self.serial_port.clone(),
            TaskPriority::NORMAL,
            turn_on,
            Some("Turning GNSS module on...".to_string()),
            (),
        )
    }

    /// Turns GNSS module off.
    pub fn turn_off(&self) -> TaskJoinHandle<()> {
        spawn_task(
            self.serial_port.clone(),
            TaskPriority::NORMAL,
            turn_off,
            Some("Turning GNSS module off...".to_string()),
            (),
        )
    }

    // Get fixed GNSS data.
    pub fn get_data(&self) -> TaskJoinHandle<GNSSData> {
        spawn_task(
            self.serial_port.clone(),
            TaskPriority::NORMAL,
            get_data,
            Some("Getting GNSS data...".to_string()),
            (),
        )
    }
}
