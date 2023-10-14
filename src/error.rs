/// returned by [`Error::kind`] method.
pub enum ErrorKind {
    GnssModuleOff,
    GnssNotFixed,
    GnssProblem,
    GprsApnConfigSetFailed,
    GprsConnectionCloseFailed,
    GprsConnectionOpenFailed,
    GprsHttpRequestFailed,
    GprsNoConnection,
    HatAlreadyOff,
    HatAlreadyOn,
    JsonSerialisationFailed,
    NotResolved,
    PhoneCallNotAnswered,
    PhoneCallNotCalled,
    PhoneCallNotEnded,
    RequestBodyParsingFailed,
    SmsNotSent,
    SmsProblemWithReadingMessages,
    SmsProblemWithSettingTextMode,
    SmsRemoveMessageFailed,
    Uart,
    UrlParse,
}

/// RPi SIM868 Error enum.
#[derive(Debug)]
pub enum Error {
    GnssModuleOff,
    GnssNotFixed,
    GnssProblem,
    GprsApnConfigSetFailed,
    GprsConnectionCloseFailed,
    GprsConnectionOpenFailed,
    GprsHttpRequestFailed,
    GprsNoConnection,
    HatAlreadyOff,
    HatAlreadyOn,
    JsonSerialisationFailed(serde_json::Error),
    NotResolved,
    PhoneCallNotAnswered,
    PhoneCallNotCalled,
    PhoneCallNotEnded,
    RequestBodyParsingFailed(serde_url_params::Error),
    SmsNotSent,
    SmsProblemWithReadingMessages,
    SmsProblemWithSettingTextMode,
    SmsRemoveMessageFailed,
    Uart(rppal::uart::Error),
    UrlParse(url::ParseError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::GnssModuleOff => write!(f, "GNSS - module is off."),
            Error::GnssNotFixed => write!(f, "GNSS - position is not fixed - check GSM antenna."),
            Error::GnssProblem => write!(f, "GNSS - problem with the module."),
            Error::GprsApnConfigSetFailed => write!(f, "GPRS - setting APN Configuration has failed."),
            Error::GprsConnectionCloseFailed => write!(f, "GPRS - closing the connection has failed."),
            Error::GprsConnectionOpenFailed => write!(f, "GPRS - opening the connection has failed. Make sure you provide valid APN configuration during sim868.gprs.init call."),
            Error::GprsHttpRequestFailed => write!(f, "GPRS - HTTP request has failed."),
            Error::GprsNoConnection => write!(f, "GPRS - no connection to the network."),
            Error::HatAlreadyOff => write!(f, "HAT - already switched off."),
            Error::HatAlreadyOn => write!(f, "HAT - already switched on."),
            Error::JsonSerialisationFailed(ref err) => write!(f, "Object has failed when serialising to JSON: {}", err),
            Error::NotResolved => write!(f, "Task NotResolved - please check if the hat is switched on."),
            Error::PhoneCallNotAnswered => write!(f, "Phone - there was an error while trying to answer the call."),
            Error::PhoneCallNotCalled => write!(f, "Phone - there was an error while trying to make a call - please check the network strength."),
            Error::PhoneCallNotEnded => write!(f, "Phone - there was an error while trying to end a call - it could end previously eg. other side has hanged up."),
            Error::RequestBodyParsingFailed(ref err) => write!(f, "Request body parsing has failed: {}", err),
            Error::SmsNotSent => write!(f, "SMS - there was an error while trying to send an SMS - please check the network strength."),
            Error::SmsProblemWithReadingMessages => write!(f, "SMS - problem with reading the messages."),
            Error::SmsProblemWithSettingTextMode => write!(f, "SMS - problem with setting the text mode."),
            Error::SmsRemoveMessageFailed => write!(f, "SMS - problem with removing the message/s."),
            Error::Uart(ref err) => write!(f, "Uart error: {}", err),
            Error::UrlParse(ref err) => write!(f, "URL parsing error: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        match self {
            Error::GnssModuleOff => ErrorKind::GnssModuleOff,
            Error::GnssNotFixed => ErrorKind::GnssNotFixed,
            Error::GnssProblem => ErrorKind::GnssProblem,
            Error::GprsApnConfigSetFailed => ErrorKind::GprsApnConfigSetFailed,
            Error::GprsConnectionCloseFailed => ErrorKind::GprsConnectionCloseFailed,
            Error::GprsConnectionOpenFailed => ErrorKind::GprsConnectionOpenFailed,
            Error::GprsHttpRequestFailed => ErrorKind::GprsHttpRequestFailed,
            Error::GprsNoConnection => ErrorKind::GprsNoConnection,
            Error::HatAlreadyOff => ErrorKind::HatAlreadyOff,
            Error::HatAlreadyOn => ErrorKind::HatAlreadyOn,
            Error::JsonSerialisationFailed(ref _e) => ErrorKind::JsonSerialisationFailed,
            Error::NotResolved => ErrorKind::NotResolved,
            Error::PhoneCallNotAnswered => ErrorKind::PhoneCallNotAnswered,
            Error::PhoneCallNotCalled => ErrorKind::PhoneCallNotCalled,
            Error::PhoneCallNotEnded => ErrorKind::PhoneCallNotEnded,
            Error::RequestBodyParsingFailed(ref _e) => ErrorKind::RequestBodyParsingFailed,
            Error::SmsNotSent => ErrorKind::SmsNotSent,
            Error::SmsProblemWithReadingMessages => ErrorKind::SmsProblemWithReadingMessages,
            Error::SmsProblemWithSettingTextMode => ErrorKind::SmsProblemWithSettingTextMode,
            Error::SmsRemoveMessageFailed => ErrorKind::SmsRemoveMessageFailed,
            Error::Uart(ref _e) => ErrorKind::Uart,
            Error::UrlParse(ref _e) => ErrorKind::UrlParse,
        }
    }
}

impl From<rppal::uart::Error> for Error {
    fn from(err: rppal::uart::Error) -> Error {
        Error::Uart(err)
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Error {
        Error::UrlParse(err)
    }
}

impl From<serde_url_params::Error> for Error {
    fn from(err: serde_url_params::Error) -> Error {
        Error::RequestBodyParsingFailed(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::JsonSerialisationFailed(err)
    }
}
