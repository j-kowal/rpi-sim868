use crate::{
    error::Error,
    error_check,
    gprs::{ContentType, Request, RequestMethod},
    serial_port::SerialPort,
    ResolverReturn, ACK_REGEX, REGEX_COMP_ERROR,
};
use regex::Regex;
use std::{sync::Arc, time::Duration};
use url::Url;
use uuid::Uuid;

fn generic_resolver(result: String, regex: &str) -> ResolverReturn<()> {
    if error_check(&result) {
        return Err(Error::GprsHttpRequestFailed);
    }
    match Regex::new(regex).expect(REGEX_COMP_ERROR).is_match(&result) {
        true => Ok(()),
        false => Err(Error::NotResolved),
    }
}

fn http_request_resolver(result: String) -> ResolverReturn<()> {
    generic_resolver(result, "\r\nOK\r\n")
}

pub fn get_content_type(content_type: &Option<ContentType>) -> String {
    if let Some(ct) = content_type {
        return match ct {
            ContentType::FormUrlencoded => "application/x-www-form-urlencoded".to_string(),
            ContentType::Json => "application/json".to_string(),
        };
    }

    // default
    "application/x-www-form-urlencoded".to_string()
}

pub fn init<T>(
    serial_port: &Arc<SerialPort>,
    task_id: &Uuid,
    request: &Request<T>,
) -> ResolverReturn<()>
where
    T: serde::Serialize,
{
    let mut url: Url = Url::parse(&request.url)?;

    if matches!(request.method, RequestMethod::GET) {
        url.set_query(Some(&serde_url_params::to_string(&request.data)?))
    }

    let mut commands = vec![
        "AT+HTTPINIT\n".to_string(),
        "AT+HTTPPARA=CID,1\n".to_string(),
        format!("AT+HTTPPARA=URL,{}\n", url),
    ];

    if let Some(headers) = &request.headers {
        commands.push(format!("AT+HTTPPARA=USERDATA,\"{}\"\n", headers))
    }

    if matches!(request.method, RequestMethod::POST) {
        commands.push(format!(
            "AT+HTTPPARA=CONTENT,{}\n",
            get_content_type(&request.content_type)
        ));
    }

    for command in commands {
        serial_port.process(task_id, command, http_request_resolver, None)?;
    }

    Ok(())
}

pub fn data<T>(
    serial_port: &Arc<SerialPort>,
    task_id: &Uuid,
    request: &Request<T>,
) -> ResolverReturn<()>
where
    T: serde::Serialize,
{
    fn http_data_resolver(result: String) -> ResolverReturn<()> {
        generic_resolver(result, "\r\nDOWNLOAD\r\n")
    }

    let content_type: ContentType = match &request.content_type {
        Some(ct) => *ct,
        None => ContentType::FormUrlencoded,
    };

    let data: String = match content_type {
        ContentType::FormUrlencoded => serde_url_params::to_string(&request.data)?,
        ContentType::Json => serde_json::to_string(&request.data)?,
    };

    serial_port.process(
        task_id,
        format!("AT+HTTPDATA={},6000\n", data.as_bytes().len()),
        http_data_resolver,
        Some(Duration::from_secs(10)),
    )?;
    serial_port.write(task_id, data)?;
    serial_port.read(task_id, http_request_resolver, Some(Duration::from_secs(6)))
}

pub fn action(
    serial_port: &Arc<SerialPort>,
    task_id: &Uuid,
    request_method: RequestMethod,
) -> ResolverReturn<()> {
    fn resolver(result: String) -> ResolverReturn<()> {
        generic_resolver(result, r"\+HTTPACTION:.*")
    }

    serial_port.process(
        task_id,
        format!("AT+HTTPACTION={}\n", request_method as u8),
        resolver,
        Some(Duration::from_secs(10)),
    )
}

pub fn read(serial_port: &Arc<SerialPort>, task_id: &Uuid) -> ResolverReturn<String> {
    fn resolver(result: String) -> ResolverReturn<String> {
        if error_check(&result) {
            return Err(Error::GprsHttpRequestFailed);
        }
        match ACK_REGEX.is_match(&result) {
            true => Ok(result),
            false => Err(Error::NotResolved),
        }
    }

    serial_port.process(
        task_id,
        "AT+HTTPREAD\n".to_string(),
        resolver,
        Some(Duration::from_secs(10)),
    )
}

pub fn terminate(serial_port: &Arc<SerialPort>, task_id: &Uuid) -> ResolverReturn<()> {
    serial_port.process(
        task_id,
        format!("AT+HTTPTERM\n"),
        http_request_resolver,
        None,
    )
}
