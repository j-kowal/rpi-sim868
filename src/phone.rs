//! Phone module
//!
//! See [`Phone`] to discover available methods.
//! # Example
//! ```
//! // This will call a number, and hang up after 20 seconds.
//! let _ = sim.phone.call("+123456789").join();
//! std::thread::sleep(time::Duration::from_secs(20));
//! let _ = sim.phone.end_call().join();
//! ```

use crate::{
    error::Error,
    generic_resolver,
    serial_port::{spawn_task, SerialPort, TaskPriority},
    Module, ResolverReturn, TaskJoinHandle, PHONE_INCOMING_CALL_REGEX,
};
use std::{sync::Arc, time::Duration};
use uuid::Uuid;

#[derive(Debug)]
pub struct IncomingCall {
    pub caller_id: String,
}

fn answer(serial_port: &Arc<SerialPort>, task_id: &Uuid, _: ()) -> ResolverReturn<()> {
    fn resolver(result: String) -> ResolverReturn<()> {
        generic_resolver(&result, Error::PhoneCallNotAnswered)
    }

    serial_port.process(task_id, "ATA\n".to_string(), resolver, None)
}

fn call(serial_port: &Arc<SerialPort>, task_id: &Uuid, number: String) -> ResolverReturn<()> {
    fn resolver(result: String) -> ResolverReturn<()> {
        generic_resolver(&result, Error::PhoneCallNotCalled)
    }

    serial_port.process(task_id, format!("ATD{number};\n"), resolver, None)
}

fn end_call(serial_port: &Arc<SerialPort>, task_id: &Uuid, _: ()) -> ResolverReturn<()> {
    fn resolver(result: String) -> ResolverReturn<()> {
        generic_resolver(&result, Error::PhoneCallNotEnded)
    }

    serial_port.process(task_id, "ATH\n".to_string(), resolver, None)
}

fn get_incoming_call(
    serial_port: &Arc<SerialPort>,
    task_id: &Uuid,
    _: (),
) -> ResolverReturn<IncomingCall> {
    fn resolver(result: String) -> ResolverReturn<IncomingCall> {
        let Some(captured) = PHONE_INCOMING_CALL_REGEX.captures(&result) else {
            return Err(Error::NotResolved);
        };

        let data: &Vec<&str> = &captured["data"].split(",").collect();
        Ok(IncomingCall {
            caller_id: data[0].replace('"', ""),
        })
    }

    serial_port.read(task_id, resolver, Some(Duration::from_secs(4)))
}

pub struct Phone {
    serial_port: Arc<SerialPort>,
}

impl Module for Phone {
    fn new(serial_port: Arc<SerialPort>) -> Self {
        Phone { serial_port }
    }
}

impl Phone {
    pub fn call(&self, number: &str) -> TaskJoinHandle<()> {
        spawn_task(
            self.serial_port.clone(),
            TaskPriority::NORMAL,
            call,
            Some(format!("Calling {number}...")),
            number.to_string(),
        )
    }

    pub fn end_call(&self) -> TaskJoinHandle<()> {
        spawn_task(
            self.serial_port.clone(),
            TaskPriority::HIGH,
            end_call,
            Some("Ending call...".to_string()),
            (),
        )
    }

    pub fn answer(&self) -> TaskJoinHandle<()> {
        spawn_task(
            self.serial_port.clone(),
            TaskPriority::HIGH,
            answer,
            Some("Ending call...".to_string()),
            (),
        )
    }

    pub fn get_incoming_call(&self) -> TaskJoinHandle<IncomingCall> {
        spawn_task(
            self.serial_port.clone(),
            TaskPriority::NORMAL,
            get_incoming_call,
            Some("Ending call...".to_string()),
            (),
        )
    }
}
