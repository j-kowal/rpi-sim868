//! SMS module
//!
//! See [`SMS`] to discover available methods.

use crate::{
    error::Error,
    error_check, generic_resolver,
    serial_port::{spawn_task, SerialPort, TaskPriority},
    Module, ResolverReturn, TaskJoinHandle, PARSING_ERROR, SMS_MESSAGE_SENT_REGEX,
    SMS_READ_MESSAGE_REGEX,
};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use std::{sync::Arc, time::Duration};
use uuid::Uuid;

fn parse_message(captured: regex::Captures<'_>) -> Message {
    let raw_data: &str = &captured["data"].to_string().trim().replace('"', "");
    let parsed_data: &Vec<&str> = &raw_data.split(",").collect();
    let raw_datetime: String = format!("{} {}", &parsed_data[3], &parsed_data[4][0..8]);
    let date_time: DateTime<Local> = TimeZone::from_local_datetime(
        &Local,
        &NaiveDateTime::parse_from_str(&raw_datetime, "%y/%m/%d %H:%M:%S").expect(PARSING_ERROR),
    )
    .unwrap();
    Message {
        index: captured["index"].parse::<u8>().expect(PARSING_ERROR),
        text: captured["text"].trim().to_string(),
        sender: parsed_data[1].to_string(),
        datetime: date_time,
    }
}

fn set_text_mode(serial_port: &Arc<SerialPort>, task_id: &Uuid) -> ResolverReturn<()> {
    fn resolver(result: String) -> ResolverReturn<()> {
        generic_resolver(&result, Error::SmsProblemWithSettingTextMode)
    }

    serial_port.process(task_id, "AT+CMGF=1\n".to_string(), resolver, None)
}

fn send(
    serial_port: &Arc<SerialPort>,
    task_id: &Uuid,
    args: (String, String),
) -> ResolverReturn<()> {
    fn resolver(result: String) -> ResolverReturn<()> {
        if error_check(&result) {
            return Err(Error::SmsNotSent);
        }
        match SMS_MESSAGE_SENT_REGEX.is_match(&result) {
            true => Ok(()),
            false => Err(Error::NotResolved),
        }
    }

    let (number, text) = args;

    set_text_mode(&serial_port, &task_id)?;
    serial_port.process(
        task_id,
        format!("AT+CMGS={number}\n{text}\x1A\n"),
        resolver,
        Some(Duration::from_secs(20)),
    )
}

fn get_messages(
    serial_port: &Arc<SerialPort>,
    task_id: &Uuid,
    storage: MessageStorage,
) -> ResolverReturn<Vec<Message>> {
    fn resolver(result: String) -> ResolverReturn<Vec<Message>> {
        let ok: Result<(), Error> = generic_resolver(&result, Error::SmsProblemWithReadingMessages);
        if let Err(err) = ok {
            return Err(err);
        }

        let messages: Vec<Message> = SMS_READ_MESSAGE_REGEX
            .captures_iter(&result)
            .map(|captured: regex::Captures<'_>| parse_message(captured))
            .collect();

        Ok(messages)
    }

    set_text_mode(&serial_port, &task_id)?;
    serial_port.process(
        task_id,
        format!(
            "AT+CMGL=\"{}\"\n",
            if matches!(storage, MessageStorage::UNREAD) {
                "REC UNREAD"
            } else {
                "ALL"
            }
        ),
        resolver,
        Some(Duration::from_secs(20)),
    )
}

fn remove_all_messages(
    serial_port: &Arc<SerialPort>,
    task_id: &Uuid,
    storage: MessageStorage,
) -> ResolverReturn<()> {
    fn resolver(result: String) -> ResolverReturn<()> {
        generic_resolver(&result, Error::SmsRemoveMessageFailed)
    }

    set_text_mode(&serial_port, &task_id)?;

    let msg_storage: &str = match storage {
        MessageStorage::ALL => "DEL ALL",
        MessageStorage::READ => "DEL READ",
        MessageStorage::UNREAD => "DEL UNREAD",
    };

    serial_port.process(
        task_id,
        format!("AT+CMGDA=\"{msg_storage}\"\n"),
        resolver,
        Some(Duration::from_secs(30)),
    )
}

fn remove_message(serial_port: &Arc<SerialPort>, task_id: &Uuid, index: u8) -> ResolverReturn<()> {
    fn resolver(result: String) -> ResolverReturn<()> {
        generic_resolver(&result, Error::SmsRemoveMessageFailed)
    }

    serial_port.process(
        task_id,
        format!("AT+CMGD={index}\n"),
        resolver,
        Some(Duration::from_secs(10)),
    )
}

#[derive(Debug)]
pub enum MessageStorage {
    UNREAD,
    READ,
    ALL,
}

#[derive(Debug)]
pub struct Message {
    pub index: u8,
    pub text: String,
    pub sender: String,
    pub datetime: DateTime<Local>,
}

pub struct SMS {
    serial_port: Arc<SerialPort>,
}

impl Module for SMS {
    fn new(serial_port: Arc<SerialPort>) -> Self {
        SMS { serial_port }
    }
}

impl SMS {
    /// Sends an SMS up to 160 characters.
    pub fn send(&self, recipient: &str, text: &str) -> TaskJoinHandle<()> {
        let number: String = format!(r#""{recipient}""#);
        spawn_task(
            self.serial_port.clone(),
            TaskPriority::NORMAL,
            send,
            Some(format!("Sending SMS to {number}: {text}")),
            (number, text.to_string()),
        )
    }

    /// Gets the messages from the given storage or ALL.
    pub fn get_messages(&self, storage: MessageStorage) -> TaskJoinHandle<Vec<Message>> {
        spawn_task(
            self.serial_port.clone(),
            TaskPriority::NORMAL,
            get_messages,
            Some("Getting messages...".to_string()),
            storage,
        )
    }

    /// Removes all messages from the selected storage or ALL.
    pub fn remove_all_messages(&self, storage: MessageStorage) -> TaskJoinHandle<()> {
        spawn_task(
            self.serial_port.clone(),
            TaskPriority::NORMAL,
            remove_all_messages,
            Some(format!("Removing all messages from {storage:?}...")),
            storage,
        )
    }

    /// Removes a single message at given index
    pub fn remove_message(&self, index: u8) -> TaskJoinHandle<()> {
        spawn_task(
            self.serial_port.clone(),
            TaskPriority::NORMAL,
            remove_message,
            Some(format!("Removing message at index: {index}...")),
            index,
        )
    }
}
