//! Unit tests for AT command response parsing

/// Test ACK (OK) pattern matching
#[test]
fn test_ack_pattern_matching() {
    use regex::Regex;
    
    let ack_regex = Regex::new(r"\r\nOK\r\n").unwrap();
    
    // Should match
    assert!(ack_regex.is_match("\r\nOK\r\n"));
    assert!(ack_regex.is_match("AT\r\n\r\nOK\r\n"));
    assert!(ack_regex.is_match("+CGNSINF: 1,1,20231015123000.000,52.2297,21.0122,100.5,0.0,0.0,1,,1.9,1.8,0.9,,,,,,,,\r\n\r\nOK\r\n"));
    
    // Should not match
    assert!(!ack_regex.is_match("\r\nERROR\r\n"));
    assert!(!ack_regex.is_match("OK"));
    assert!(!ack_regex.is_match("\r\n"));
}

/// Test ERROR pattern matching
#[test]
fn test_error_pattern_matching() {
    use regex::Regex;
    
    let error_regex = Regex::new(r"\r\nERROR\r\n").unwrap();
    
    // Should match
    assert!(error_regex.is_match("\r\nERROR\r\n"));
    assert!(error_regex.is_match("AT+CMGS=\"+123456789\"\r\n\r\nERROR\r\n"));
    
    // Should not match
    assert!(!error_regex.is_match("\r\nOK\r\n"));
    assert!(!error_regex.is_match("ERROR"));
}

/// Test GNSS data parsing with CGNSINF response
#[test]
fn test_gnss_data_parsing() {
    use regex::Regex;
    
    let gnss_regex = Regex::new(r"\+CGNSINF: (?<data>.+)").unwrap();
    
    let response = "+CGNSINF: 1,1,20231015123000.000,52.2297,21.0122,100.5,0.0,0.0,1,,1.9,1.8,0.9,,,,,,,,\r\n";
    
    let captures = gnss_regex.captures(response).unwrap();
    let data = captures.name("data").unwrap().as_str();
    
    // Verify extracted data contains expected fields
    let fields: Vec<&str> = data.split(',').collect();
    assert_eq!(fields.len(), 21); // CGNSINF has 21 fields
    assert_eq!(fields[0], "1"); // GNSS run status
    assert_eq!(fields[1], "1"); // Fix status
    assert_eq!(fields[2], "20231015123000.000"); // UTC datetime
    assert_eq!(fields[3], "52.2297"); // Latitude
    assert_eq!(fields[4], "21.0122"); // Longitude
}

/// Test GNSS power status parsing
#[test]
fn test_gnss_power_parsing() {
    use regex::Regex;
    
    let power_regex = Regex::new(r"\+CGNSPWR: (?<number>\d)").unwrap();
    
    // Power ON
    let on_response = "+CGNSPWR: 1\r\n";
    let captures = power_regex.captures(on_response).unwrap();
    let status = captures.name("number").unwrap().as_str();
    assert_eq!(status, "1");
    
    // Power OFF
    let off_response = "+CGNSPWR: 0\r\n";
    let captures = power_regex.captures(off_response).unwrap();
    let status = captures.name("number").unwrap().as_str();
    assert_eq!(status, "0");
}

/// Test signal strength parsing
#[test]
fn test_signal_strength_parsing() {
    use regex::Regex;
    
    let signal_regex = Regex::new(r"\+CSQ: (?<number>\d*)").unwrap();
    
    // Good signal
    let good = "+CSQ: 22\r\n";
    let captures = signal_regex.captures(good).unwrap();
    let strength = captures.name("number").unwrap().as_str();
    assert_eq!(strength, "22");
    
    // No signal
    let no_signal = "+CSQ: 0\r\n";
    let captures = signal_regex.captures(no_signal).unwrap();
    let strength = captures.name("number").unwrap().as_str();
    assert_eq!(strength, "0");
    
    // Unknown (empty)
    let unknown = "+CSQ: \r\n";
    let captures = signal_regex.captures(unknown).unwrap();
    let strength = captures.name("number").unwrap().as_str();
    assert_eq!(strength, "");
}

/// Test GPRS connection status parsing
#[test]
fn test_gprs_status_parsing() {
    use regex::Regex;
    
    let gprs_regex = Regex::new(r"\+SAPBR: (?<data>.+)").unwrap();
    
    let response = "+SAPBR: 1,1,\"10.0.0.1\"\r\n";
    let captures = gprs_regex.captures(response).unwrap();
    let data = captures.name("data").unwrap().as_str();
    
    let fields: Vec<&str> = data.split(',').collect();
    assert_eq!(fields[0], "1"); // Bearer profile ID
    assert_eq!(fields[1], "1"); // Connect status (0=closed, 1=open)
}

/// Test phone incoming call parsing (CLIP)
#[test]
fn test_phone_incoming_call_parsing() {
    use regex::Regex;
    
    let clip_regex = Regex::new(r"\+CLIP: (?<data>.+)").unwrap();
    
    let response = r#"+CLIP: "+4799999999",145,,,,0"#;
    let captures = clip_regex.captures(response).unwrap();
    let data = captures.name("data").unwrap().as_str();
    
    let fields: Vec<&str> = data.split(',').collect();
    assert_eq!(fields[0], r#""+4799999999""#); // Phone number
    assert_eq!(fields[1], "145"); // Type (145=international)
}

/// Test SMS message sent confirmation
#[test]
fn test_sms_sent_parsing() {
    use regex::Regex;
    
    let sms_regex = Regex::new(r"\+CMGS: \d").unwrap();
    
    let response = "+CMGS: 1\r\n";
    assert!(sms_regex.is_match(response));
    
    let response2 = "+CMGS: 42\r\n";
    assert!(sms_regex.is_match(response2));
    
    // Should not match
    assert!(!sms_regex.is_match("+CMGS:\r\n"));
}

/// Test SMS read message parsing
#[test]
fn test_sms_read_parsing() {
    use regex::Regex;
    
    let sms_regex = Regex::new(r"\+CMGL: (?<index>\d*),(?<data>.+)\r\n(?<text>.+)").unwrap();
    
    let response = "+CMGL: 1,\"REC READ\",\"+4799999999\",,\"23/10/15,12:30:00+08\"\r\nHello World\r\n";
    
    let captures = sms_regex.captures(response).unwrap();
    let index = captures.name("index").unwrap().as_str();
    let data = captures.name("data").unwrap().as_str();
    let text = captures.name("text").unwrap().as_str().trim_end_matches('\r');
    
    assert_eq!(index, "1");
    assert!(data.contains("REC READ"));
    assert!(data.contains("+4799999999"));
    assert_eq!(text, "Hello World");
}

/// Test complex multi-line response parsing
#[test]
fn test_multiline_response_parsing() {
    use regex::Regex;
    
    // Simulate AT+CGNSINF response with multiple lines
    let response = r#"AT+CGNSINF

+CGNSINF: 1,1,20231015123000.000,52.2297,21.0122,100.5,0.0,0.0,1,,1.9,1.8,0.9,,,,,,,,

OK
"#;
    
    // Use (?s) flag to make . match newlines
    let gnss_regex = Regex::new(r"(?s)\+CGNSINF: (?<data>.+)\r?\n\r?\nOK").unwrap();
    
    assert!(gnss_regex.is_match(response));
    
    let captures = gnss_regex.captures(response).unwrap();
    let data = captures.name("data").unwrap().as_str().trim_end();
    assert!(data.contains("52.2297"));
    assert!(data.contains("21.0122"));
}

/// Test parsing with error responses
#[test]
fn test_error_response_parsing() {
    use regex::Regex;
    
    let error_regex = Regex::new(r"\r\nERROR\r\n").unwrap();
    let cms_error_regex = Regex::new(r"\+CMS ERROR: \d+").unwrap();
    let cme_error_regex = Regex::new(r"\+CME ERROR: \d+").unwrap();
    
    // Standard ERROR
    assert!(error_regex.is_match("AT+CMGS=\"invalid\r\n\r\nERROR\r\n"));
    
    // CMS ERROR (SMS related)
    assert!(cms_error_regex.is_match("+CMS ERROR: 305"));
    assert!(cms_error_regex.is_match("+CMS ERROR: 500"));
    
    // CME ERROR (ME related)
    assert!(cme_error_regex.is_match("+CME ERROR: 10"));
    assert!(cme_error_regex.is_match("+CME ERROR: 100"));
}

/// Test parsing of network registration status
#[test]
fn test_network_registration_parsing() {
    use regex::Regex;
    
    let creg_regex = Regex::new(r"\+CREG: (?<data>.+)").unwrap();
    
    let response = "+CREG: 2,1,\"00FF\",\"12345678\"\r\n";
    let captures = creg_regex.captures(response).unwrap();
    let data = captures.name("data").unwrap().as_str();
    
    let fields: Vec<&str> = data.split(',').collect();
    assert_eq!(fields[0], "2"); // Mode
    assert_eq!(fields[1], "1"); // Status (1=registered, home network)
}

/// Test parsing of battery status
#[test]
fn test_battery_status_parsing() {
    use regex::Regex;
    
    let cbc_regex = Regex::new(r"\+CBC: (?<data>.+)").unwrap();
    
    let response = "+CBC: 0,85,4200\r\n";
    let captures = cbc_regex.captures(response).unwrap();
    let data = captures.name("data").unwrap().as_str().trim_end_matches('\r');
    
    let fields: Vec<&str> = data.split(',').collect();
    assert_eq!(fields[0], "0"); // Battery charge status
    assert_eq!(fields[1], "85"); // Battery level (percentage)
    assert_eq!(fields[2], "4200"); // Battery voltage (mV)
}

/// Test parsing of HTTP response
#[test]
fn test_http_response_parsing() {
    use regex::Regex;
    
    let http_regex = Regex::new(r"\+HTTPACTION: (?<data>.+)").unwrap();
    
    let response = "+HTTPACTION: 0,200,1234\r\n";
    let captures = http_regex.captures(response).unwrap();
    let data = captures.name("data").unwrap().as_str().trim_end_matches('\r');
    
    let fields: Vec<&str> = data.split(',').collect();
    assert_eq!(fields[0], "0"); // Method (0=GET)
    assert_eq!(fields[1], "200"); // Status code
    assert_eq!(fields[2], "1234"); // Data length
}

/// Test edge cases in parsing
#[test]
fn test_parsing_edge_cases() {
    use regex::Regex;
    
    let gnss_regex = Regex::new(r"\+CGNSINF: (?<data>.+)").unwrap();
    
    // Empty data field
    let empty_response = "+CGNSINF: ,,,,,,,,,,,,,,,,,,\r\n";
    let captures = gnss_regex.captures(empty_response).unwrap();
    let data = captures.name("data").unwrap().as_str();
    assert!(data.contains(','));
    
    // Partial data
    let partial = "+CGNSINF: 1,0,,,,,,,,,,,,,,,,\r\n";
    let captures = gnss_regex.captures(partial).unwrap();
    let data = captures.name("data").unwrap().as_str();
    assert!(data.starts_with("1,0"));
}

/// Test parsing with special characters in SMS text
#[test]
fn test_sms_special_characters_parsing() {
    use regex::Regex;
    
    let sms_regex = Regex::new(r"\+CMGL: (?<index>\d*),(?<data>.+)\r\n(?<text>.+)").unwrap();
    
    // SMS with special characters
    let response = "+CMGL: 1,\"REC READ\",\"+4799999999\",,\"23/10/15,12:30:00+08\"\r\nHello! How are you? :) #test\r\n";
    
    let captures = sms_regex.captures(response).unwrap();
    let text = captures.name("text").unwrap().as_str().trim_end_matches('\r');
    assert_eq!(text, "Hello! How are you? :) #test");
}
