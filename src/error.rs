#[derive(Copy, Clone, Debug)]
#[repr(u32)]
pub enum NodeWatcherErrorType {
    Unknown = 0,
}

pub struct NodeWatcherError {
    pub error_type: NodeWatcherErrorType,
    pub reason: Option<String>,
}

impl NodeWatcherError {
    pub fn new(error_type: NodeWatcherErrorType) -> Self {
        Self {
            error_type,
            reason: None,
        }
    }

    pub fn new_with_reason(error_type: NodeWatcherErrorType, reason: &str) -> Self {
        Self {
            error_type,
            reason: Some(String::from(reason)),
        }
    }
}

// TODO: Return more useful errors...
impl From<notify::Error> for NodeWatcherError {
    #[inline]
    fn from(e: notify::Error) -> NodeWatcherError {
        NodeWatcherError::new_with_reason(NodeWatcherErrorType::Unknown, "watcher error")
    }
}

impl From<NodeWatcherError> for napi::Error {
    #[inline]
    fn from(err: NodeWatcherError) -> napi::Error {
        // Prefix all errors, so it's obvious they originate from this library
        let mut reason = String::from("[watcher] ");

        // Convert error type into an error message...
        match err.error_type {
            NodeWatcherErrorType::Unknown => {
                reason.push_str("Unknown error");
            }
        }

        // Add reason to error string if there is one
        if let Some(r) = err.reason {
            reason.push_str(", ");
            reason.push_str(&r[..]);
        }

        // Return a napi error
        napi::Error::new(napi::Status::GenericFailure, reason)
    }
}
