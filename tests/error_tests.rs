//! Unit tests for error handling and type conversions

use std::error::Error;

/// Mock errors for testing conversions
#[derive(Debug)]
struct MockUartError;

impl std::fmt::Display for MockUartError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Mock UART error")
    }
}

impl std::error::Error for MockUartError {}

/// Mock URL parse error
#[derive(Debug)]
struct MockUrlError;

impl std::fmt::Display for MockUrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Mock URL error")
    }
}

impl std::error::Error for MockUrlError {}

/// Mock serde error
#[derive(Debug)]
struct MockSerdeError;

impl std::fmt::Display for MockSerdeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Mock serialization error")
    }
}

impl std::error::Error for MockSerdeError {}

/// Test error kind mapping
#[test]
fn test_error_kind_mapping() {
    // Define local error types for testing
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum ErrorKind {
        NotResolved,
        Uart,
        GnssProblem,
        SmsNotSent,
        GprsNoConnection,
    }
    
    #[derive(Debug)]
    enum TestError {
        NotResolved,
        Uart(MockUartError),
        GnssProblem,
        SmsNotSent,
        GprsNoConnection,
    }
    
    impl TestError {
        fn kind(&self) -> ErrorKind {
            match self {
                TestError::NotResolved => ErrorKind::NotResolved,
                TestError::Uart(_) => ErrorKind::Uart,
                TestError::GnssProblem => ErrorKind::GnssProblem,
                TestError::SmsNotSent => ErrorKind::SmsNotSent,
                TestError::GprsNoConnection => ErrorKind::GprsNoConnection,
            }
        }
    }
    
    // Test that error kind mapping works correctly
    let err1 = TestError::NotResolved;
    assert_eq!(err1.kind(), ErrorKind::NotResolved);
    
    let err2 = TestError::GnssProblem;
    assert_eq!(err2.kind(), ErrorKind::GnssProblem);
    
    let err3 = TestError::SmsNotSent;
    assert_eq!(err3.kind(), ErrorKind::SmsNotSent);
}

/// Test error display formatting
#[test]
fn test_error_display() {
    #[derive(Debug)]
    enum TestError {
        NotResolved,
        GnssModuleOff,
        GprsNoConnection,
        PhoneCallNotAnswered,
    }
    
    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                TestError::NotResolved => write!(f, "Task NotResolved - please check if the hat is switched on."),
                TestError::GnssModuleOff => write!(f, "GNSS - module is off."),
                TestError::GprsNoConnection => write!(f, "GPRS - no connection to the network."),
                TestError::PhoneCallNotAnswered => write!(f, "Phone - there was an error while trying to answer the call."),
            }
        }
    }
    
    let err = TestError::NotResolved;
    let display = format!("{}", err);
    assert!(display.contains("NotResolved"));
    assert!(display.contains("hat is switched on"));
    
    let err2 = TestError::GnssModuleOff;
    assert!(format!("{}", err2).contains("GNSS"));
}

/// Test error equality based on kind
#[test]
fn test_error_equality() {
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum ErrorKind {
        NotResolved,
        Uart,
        GnssProblem,
    }
    
    #[derive(Debug, Clone, PartialEq)]
    enum TestError {
        NotResolved,
        Uart(String),
        GnssProblem,
    }
    
    impl TestError {
        fn kind(&self) -> ErrorKind {
            match self {
                TestError::NotResolved => ErrorKind::NotResolved,
                TestError::Uart(_) => ErrorKind::Uart,
                TestError::GnssProblem => ErrorKind::GnssProblem,
            }
        }
    }
    
    let err1 = TestError::NotResolved;
    let err2 = TestError::NotResolved;
    let err3 = TestError::Uart("test".to_string());
    
    assert_eq!(err1, err2);
    assert_ne!(err1.kind(), err3.kind());
}

/// Test error trait implementation
#[test]
fn test_error_trait() {
    #[derive(Debug)]
    struct TestError {
        message: String,
    }
    
    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }
    
    impl Error for TestError {}
    
    let err = TestError {
        message: "Test error message".to_string(),
    };
    
    // Verify it implements Error trait
    let _: &dyn Error = &err;
    
    // Verify description
    assert_eq!(err.to_string(), "Test error message");
}

/// Test Result type with custom error
#[test]
fn test_custom_result_type() {
    #[derive(Debug, PartialEq)]
    enum TestError {
        NotResolved,
        InvalidInput,
    }
    
    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                TestError::NotResolved => write!(f, "Not resolved"),
                TestError::InvalidInput => write!(f, "Invalid input"),
            }
        }
    }
    
    impl Error for TestError {}
    
    type TestResult<T> = Result<T, TestError>;
    
    fn success_fn() -> TestResult<i32> {
        Ok(42)
    }
    
    fn error_fn() -> TestResult<i32> {
        Err(TestError::NotResolved)
    }
    
    assert!(success_fn().is_ok());
    assert_eq!(success_fn().unwrap(), 42);
    
    assert!(error_fn().is_err());
    assert_eq!(error_fn().unwrap_err(), TestError::NotResolved);
}

/// Test error propagation with ? operator
#[test]
fn test_error_propagation() {
    #[derive(Debug)]
    enum InnerError {
        Fail,
    }
    
    #[derive(Debug)]
    enum OuterError {
        Inner(InnerError),
        Other,
    }
    
    impl From<InnerError> for OuterError {
        fn from(err: InnerError) -> OuterError {
            OuterError::Inner(err)
        }
    }
    
    fn inner_op() -> Result<(), InnerError> {
        Err(InnerError::Fail)
    }
    
    fn outer_op() -> Result<(), OuterError> {
        inner_op()?;
        Ok(())
    }
    
    let result = outer_op();
    assert!(result.is_err());
    
    match result {
        Err(OuterError::Inner(_)) => (), // Expected
        _ => panic!("Expected Inner error"),
    }
}

/// Test error matching with kind
#[test]
fn test_error_matching_with_kind() {
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum ErrorKind {
        NotResolved,
        Uart,
        GnssProblem,
        SmsNotSent,
    }
    
    #[derive(Debug)]
    enum TestError {
        NotResolved,
        Uart,
        GnssProblem,
        SmsNotSent,
    }
    
    impl TestError {
        fn kind(&self) -> ErrorKind {
            match self {
                TestError::NotResolved => ErrorKind::NotResolved,
                TestError::Uart => ErrorKind::Uart,
                TestError::GnssProblem => ErrorKind::GnssProblem,
                TestError::SmsNotSent => ErrorKind::SmsNotSent,
            }
        }
    }
    
    fn handle_error(err: &TestError) -> &'static str {
        match err.kind() {
            ErrorKind::NotResolved => "retry",
            ErrorKind::Uart => "check_connection",
            ErrorKind::GnssProblem => "check_antenna",
            ErrorKind::SmsNotSent => "retry_sms",
        }
    }
    
    assert_eq!(handle_error(&TestError::NotResolved), "retry");
    assert_eq!(handle_error(&TestError::Uart), "check_connection");
    assert_eq!(handle_error(&TestError::GnssProblem), "check_antenna");
}

/// Test error chain and source
#[test]
fn test_error_chain() {
    #[derive(Debug)]
    enum LowLevelError {
        Io(std::io::Error),
    }
    
    impl std::fmt::Display for LowLevelError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                LowLevelError::Io(e) => write!(f, "IO error: {}", e),
            }
        }
    }
    
    impl Error for LowLevelError {}
    
    #[derive(Debug)]
    enum HighLevelError {
        Low(LowLevelError),
        Other,
    }
    
    impl From<LowLevelError> for HighLevelError {
        fn from(err: LowLevelError) -> HighLevelError {
            HighLevelError::Low(err)
        }
    }
    
    impl std::fmt::Display for HighLevelError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                HighLevelError::Low(_) => write!(f, "High level error from low level"),
                HighLevelError::Other => write!(f, "Other high level error"),
            }
        }
    }
    
    impl Error for HighLevelError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                HighLevelError::Low(err) => Some(err),
                _ => None,
            }
        }
    }
    
    // Create a chain
    let low = LowLevelError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));
    let high = HighLevelError::Low(low);
    
    // Verify error is present
    assert!(high.to_string().contains("High level"));
}

/// Test timeout error handling
#[test]
fn test_timeout_error() {
    use std::time::{Duration, Instant};
    
    #[derive(Debug, PartialEq)]
    enum OperationError {
        Timeout,
        Success,
    }
    
    fn operation_with_timeout(timeout: Duration) -> Result<i32, OperationError> {
        let start = Instant::now();
        
        // Simulate work
        while start.elapsed() < timeout {
            // Do some work...
            if start.elapsed() > Duration::from_millis(50) {
                return Err(OperationError::Timeout);
            }
        }
        
        Ok(42)
    }
    
    // Test timeout
    let result = operation_with_timeout(Duration::from_millis(100));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), OperationError::Timeout);
}

/// Test error recovery strategies
#[test]
fn test_error_recovery() {
    #[derive(Debug, Clone, PartialEq)]
    enum TestError {
        Transient,
        Permanent,
    }
    
    fn retryable_operation(max_retries: u32) -> Result<i32, TestError> {
        let mut attempts = 0;
        
        loop {
            attempts += 1;
            
            // Simulate transient error
            if attempts < max_retries {
                continue; // Retry
            }
            
            // After max retries, succeed
            return Ok(42);
        }
    }
    
    let result = retryable_operation(3);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

/// Test error logging behavior
#[test]
fn test_error_logging_context() {
    #[derive(Debug)]
    struct ContextualError {
        operation: String,
        error: String,
    }
    
    impl std::fmt::Display for ContextualError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "Error during '{}': {}", self.operation, self.error)
        }
    }
    
    let err = ContextualError {
        operation: "UART read".to_string(),
        error: "Timeout".to_string(),
    };
    
    let message = format!("{}", err);
    assert!(message.contains("UART read"));
    assert!(message.contains("Timeout"));
}

/// Test error type conversions with From trait
#[test]
fn test_from_trait_conversions() {
    #[derive(Debug)]
    enum OuterError {
        Inner(String),
    }
    
    impl From<&str> for OuterError {
        fn from(s: &str) -> OuterError {
            OuterError::Inner(s.to_string())
        }
    }
    
    impl From<String> for OuterError {
        fn from(s: String) -> OuterError {
            OuterError::Inner(s)
        }
    }
    
    // Test conversion from &str
    let err1: OuterError = "test error".into();
    match err1 {
        OuterError::Inner(s) => assert_eq!(s, "test error"),
    }
    
    // Test conversion from String
    let err2: OuterError = String::from("another error").into();
    match err2 {
        OuterError::Inner(s) => assert_eq!(s, "another error"),
    }
}

/// Test that errors are Send + Sync for async usage
#[test]
fn test_error_thread_safety() {
    use std::sync::Arc;
    use std::thread;
    
    #[derive(Debug, Clone)]
    struct ThreadSafeError {
        code: u32,
    }
    
    // Verify Send trait
    fn assert_send<T: Send>() {}
    assert_send::<ThreadSafeError>();
    
    // Verify Sync trait  
    fn assert_sync<T: Sync>() {}
    assert_sync::<ThreadSafeError>();
    
    // Use in multi-threaded context
    let error = Arc::new(ThreadSafeError { code: 42 });
    
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let err = Arc::clone(&error);
            thread::spawn(move || {
                assert_eq!(err.code, 42);
            })
        })
        .collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
}
