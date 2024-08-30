use crate::types::proto;

impl From<String> for proto::LogFilter {
    fn from(log_filter: String) -> proto::LogFilter {
        proto::LogFilter { log_filter }
    }
}

impl From<proto::LogFilter> for String {
    fn from(value: proto::LogFilter) -> Self {
        value.log_filter
    }
}
