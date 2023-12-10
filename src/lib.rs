//! # RPi SIM868
//!
//! RPi SIM868 is a Rust crate designed to simplify interaction with the [Waveshare SIM868 HAT](https://www.waveshare.com/gsm-gprs-gnss-hat.htm) for Raspberry Pi.
//! It utilizes the [`tokio`] runtime for managing asynchronous tasks and includes its own task scheduler based on a priority queue.
//! Each method call initiates a new task, which is enqueued with a priority to ensure swift execution as soon as the serial port becomes available.
//!
//! Methods (except for [`hat::Hat::turn_on`]) return [`TaskJoinHandle<T>`], where `T` represents the type resulting from parsing and analyzing the serial output, if applicable.
//! Tasks related to phone calls are treated as first-class citizens with high priority, reducing delays in answering or concluding calls.
//!
//! RPi SIM868 was conceived following a high-altitude balloon launch where the HAT served as a backup tracking device.
//! The initial software, written in Python, lacked the performance and safety synonymous with Rust.
//!
//! **Tested SIM868 UART selection switch:** A - `ttyUSBx` port, and B - `ttySx` port.
//!
//! **Tested devices:** RPi 3 Model B, RPi 4 Model B, RPi Zero W, RPi Zero 2 W.
//!
//! ## Example usage
//! ```
//! use rpi_sim868::{SIM868, TaskJoinHandle};
//! use tokio::time::sleep;
//! use std::time::Duration;
//!
//! #[tokio:main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let sim: SIM868 = SIM868::new("/dev/ttyS0", 115200, rpi_sim868::LogLevelFilter::Error);
//!
//!     sim.hat.turn_on().await?;
//!
//!     // waiting for the GSM network connection...
//!     while let Ok(strength) = sim.hat.network_strength().await? {
//!         if strength > 0 {
//!             break;
//!         }
//!         sleep(Duration::from_secs(2)).await;
//!     }
//!
//!     // task is spawned by tokio::spawn and starts in the background
//!     let send_sms: TaskJoinHandle<()> = sim.sms.send("+4799999999", "Hello!");
//!
//!     /*
//!         Some other operations...
//!     */
//!
//!     // the .await? returns the task Result or errors with tokio::task::JoinError
//!     match send_sms.await? {
//!         Ok(_) => println!("the SMS has been sent."),
//!         Err(e) => println!("Problem with sending the SMS: {e:?}"),
//!     }
//!
//!     sim.hat.turn_off().await??;
//!
//!     Ok(())
//! }
//! ```

#![doc(html_root_url = "https://docs.rs/rpi_sim868/0.1.1")]

pub mod gnss;
pub mod gprs;
pub mod hat;
pub mod phone;
pub mod sms;

mod error;
mod http;
mod serial_port;

pub use error::{Error, ErrorKind};
pub use log::LevelFilter as LogLevelFilter;

use lazy_static::lazy_static;
use regex::Regex;
use simple_logger::SimpleLogger;
use std::sync::Arc;
use tokio::task::JoinHandle;

/// Every method, except [`hat::Hat::turn_on`] (which is blocking), returns a `TaskJoinHandle<T>`.
pub type TaskJoinHandle<T> = JoinHandle<Result<T, error::Error>>;

const REGEX_COMP_ERROR: &str = "Critical error: Regex compilation has failed.";
const PARSING_ERROR: &str =
    "Critical error: Parsing of the value which suppose to produce no errors has failed.";

lazy_static! {
    static ref ACK_REGEX: Regex = Regex::new("\r\nOK\r\n").expect(REGEX_COMP_ERROR);
    static ref ERROR_REGEX: Regex = Regex::new("\r\nERROR\r\n").expect(REGEX_COMP_ERROR);
    static ref GNSS_DATA_REGEX: Regex =
        Regex::new(r"\+CGNSINF: (?<data>.+)").expect(REGEX_COMP_ERROR);
    static ref GNSS_POWER_REGEX: Regex =
        Regex::new(r"\+CGNSPWR: (?<number>\d)").expect(REGEX_COMP_ERROR);
    static ref GPRS_CONN_STATUS_REGEX: Regex =
        Regex::new(r"\+SAPBR: (?<data>.+)").expect(REGEX_COMP_ERROR);
    static ref HAT_SIGNAL_STRENGHT_REGEX: Regex =
        Regex::new(r"\+CSQ: (?<number>\d*)").expect(REGEX_COMP_ERROR);
    static ref PHONE_INCOMING_CALL_REGEX: Regex =
        Regex::new(r"\+CLIP: (?<data>.+)").expect(REGEX_COMP_ERROR);
    static ref SMS_READ_MESSAGE_REGEX: Regex =
        Regex::new(r"\+CMGL: (?<index>\d*),(?<data>.+)\r\n(?<text>.+)").expect(REGEX_COMP_ERROR);
    static ref SMS_MESSAGE_SENT_REGEX: Regex = Regex::new(r"\+CMGS: \d").expect(REGEX_COMP_ERROR);
}

type ResolverReturn<T> = Result<T, error::Error>;
trait Module {
    fn new(serial_port: Arc<serial_port::SerialPort>) -> Self;
}

fn ack_check(text: &str) -> bool {
    ACK_REGEX.is_match(text)
}

fn error_check(text: &str) -> bool {
    ERROR_REGEX.is_match(text)
}

fn generic_resolver(result: &str, err: error::Error) -> ResolverReturn<()> {
    if error_check(&result) {
        return Err(err);
    }
    match ack_check(&result) {
        true => Ok(()),
        false => Err(error::Error::NotResolved),
    }
}

pub struct SIM868 {
    pub hat: hat::Hat,
    pub sms: sms::SMS,
    pub gnss: gnss::GNSS,
    pub phone: phone::Phone,
    pub gprs: gprs::GPRS,
}

impl SIM868 {
    pub fn new(path: &str, baud_rate: u32, log_level: LogLevelFilter) -> Self {
        match log_level {
            LogLevelFilter::Off => (),
            _ => SimpleLogger::new()
                .with_level(log_level)
                .init()
                .expect("Problems with initialising the logger."),
        }

        let serial_port: Arc<serial_port::SerialPort> =
            Arc::new(serial_port::SerialPort::new(path, baud_rate));

        SIM868 {
            gnss: gnss::GNSS::new(serial_port.clone()),
            hat: hat::Hat::new(serial_port.clone()),
            sms: sms::SMS::new(serial_port.clone()),
            gprs: gprs::GPRS::new(serial_port.clone()),
            phone: phone::Phone::new(serial_port),
        }
    }
}
