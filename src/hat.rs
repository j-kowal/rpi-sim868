//! Hat module
//!
//! See [`Hat`] to discover available methods.

use crate::{
    ack_check,
    error::{Error, ErrorKind},
    serial_port::{spawn_task, SerialPort, TaskPriority},
    Module, ResolverReturn, TaskJoinHandle, HAT_SIGNAL_STRENGHT_REGEX, PARSING_ERROR,
};
use rppal::gpio::{Gpio, OutputPin};
use std::{sync::Arc, thread::sleep, time::Duration};
use uuid::Uuid;

const TOGGLE_POWER_PIN: u8 = 4;

pub struct Hat {
    serial_port: Arc<SerialPort>,
}

fn is_on(serial_port: &Arc<SerialPort>, task_id: &Uuid, _: ()) -> ResolverReturn<bool> {
    fn resolver(result: String) -> ResolverReturn<bool> {
        match ack_check(&result) {
            true => Ok(true),
            false => Err(Error::NotResolved),
        }
    }

    serial_port.process(
        task_id,
        "AT\n".to_string(),
        resolver,
        Some(Duration::from_secs(2)),
    )
}

fn turn_off(serial_port: &Arc<SerialPort>, task_id: &Uuid, _: ()) -> ResolverReturn<()> {
    match is_on(serial_port, task_id, ()) {
        Ok(_) => serial_port.write(task_id, "AT+CPOWD=0\n".to_string()),
        Err(e) => {
            if matches!(e.kind(), ErrorKind::NotResolved) {
                Err(Error::HatAlreadyOff)
            } else {
                Err(e)
            }
        }
    }
}

fn network_strength(serial_port: &Arc<SerialPort>, task_id: &Uuid, _: ()) -> ResolverReturn<u8> {
    fn resolver(result: String) -> ResolverReturn<u8> {
        match HAT_SIGNAL_STRENGHT_REGEX.captures(&result) {
            Some(captured) => Ok(captured["number"].parse().expect(PARSING_ERROR)),
            None => Err(Error::NotResolved),
        }
    }

    serial_port.process(task_id, "AT+CSQ\n".to_string(), resolver, None)
}

impl Module for Hat {
    fn new(serial_port: Arc<SerialPort>) -> Self {
        Hat { serial_port }
    }
}

impl Hat {
    fn toggle_power(&self) {
        let mut toggle_power_pin: OutputPin = Gpio::new()
            .expect("Can't connect to GPIO")
            .get(TOGGLE_POWER_PIN)
            .expect(format!("Can't connect to the GPIO {TOGGLE_POWER_PIN} pin").as_str())
            .into_output();
        toggle_power_pin.set_low();
        sleep(Duration::from_millis(4000));
        toggle_power_pin.set_high();
    }

    pub fn is_on(&self) -> TaskJoinHandle<bool> {
        spawn_task(
            self.serial_port.clone(),
            TaskPriority::NORMAL,
            is_on,
            Some("Checking hat status...".to_string()),
            (),
        )
    }

    pub fn network_strength(&self) -> TaskJoinHandle<u8> {
        spawn_task(
            self.serial_port.clone(),
            TaskPriority::NORMAL,
            network_strength,
            Some("Checking network strength...".to_string()),
            (),
        )
    }

    /// Turns on the HAT (only if connected to the GPIO pin).
    pub async fn turn_on(&self) -> ResolverReturn<()> {
        match self.is_on().await? {
            Ok(_) => Err(Error::HatAlreadyOn),
            Err(e) => match e.kind() {
                ErrorKind::NotResolved => {
                    log::info!("Turning SIM868 hat on...");
                    self.toggle_power();
                    Ok(())
                }
                _ => Err(e),
            },
        }
    }

    /// Turns off the HAT.
    pub fn turn_off(&self) -> TaskJoinHandle<()> {
        spawn_task(
            self.serial_port.clone(),
            TaskPriority::HIGH,
            turn_off,
            Some("Turning SIM868 hat off...".to_string()),
            (),
        )
    }
}
