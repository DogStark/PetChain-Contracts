#[derive(Debug, Clone, PartialEq)]
pub struct TraceContext {
    pub version: String,
    pub trace_id: String,
    pub parent_span_id: String,
    pub trace_flags: String,
}

impl TraceContext {
    pub fn parse(header: &str) -> Result<Self, String> {
        let parts: Vec<&str> = header.split('-').collect();
        if parts.len() != 4 {
            return Err("invalid traceparent: expected 4 segments".to_string());
        }

        let (version, trace_id, parent_span_id, trace_flags) =
            (parts[0], parts[1], parts[2], parts[3]);

        if version.len() != 2
            || trace_id.len() != 32
            || parent_span_id.len() != 16
            || trace_flags.len() != 2
        {
            return Err("invalid traceparent segment length".to_string());
        }

        if !version.chars().all(|c| c.is_ascii_hexdigit())
            || !trace_id.chars().all(|c| c.is_ascii_hexdigit())
            || !parent_span_id.chars().all(|c| c.is_ascii_hexdigit())
            || !trace_flags.chars().all(|c| c.is_ascii_hexdigit())
        {
            return Err("traceparent segments must be valid hex".to_string());
        }

        if version != "00" {
            return Err(format!("unsupported traceparent version: {}", version));
        }

        if trace_id == "00000000000000000000000000000000" {
            return Err("trace_id must not be all zeros".to_string());
        }

        if parent_span_id == "0000000000000000" {
            return Err("parent_span_id must not be all zeros".to_string());
        }

        Ok(TraceContext {
            version: version.to_string(),
            trace_id: trace_id.to_string(),
            parent_span_id: parent_span_id.to_string(),
            trace_flags: trace_flags.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_header() {
        let ctx =
            TraceContext::parse("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01").unwrap();
        assert_eq!(ctx.version, "00");
        assert_eq!(ctx.trace_id, "4bf92f3577b34da6a3ce929d0e0e4736");
        assert_eq!(ctx.parent_span_id, "00f067aa0ba902b7");
        assert_eq!(ctx.trace_flags, "01");
    }

    #[test]
    fn rejects_non_zero_version() {
        let err =
            TraceContext::parse("01-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01")
                .unwrap_err();
        assert!(err.contains("unsupported traceparent version"));
    }

    #[test]
    fn rejects_all_zero_trace_id() {
        let err =
            TraceContext::parse("00-00000000000000000000000000000000-00f067aa0ba902b7-01")
                .unwrap_err();
        assert!(err.contains("trace_id must not be all zeros"));
    }

    #[test]
    fn rejects_all_zero_parent_span_id() {
        let err =
            TraceContext::parse("00-4bf92f3577b34da6a3ce929d0e0e4736-0000000000000000-01")
                .unwrap_err();
        assert!(err.contains("parent_span_id must not be all zeros"));
    }

    #[test]
    fn rejects_wrong_segment_count() {
        assert!(TraceContext::parse("00-abc-01").is_err());
    }

    #[test]
    fn rejects_invalid_hex() {
        let err =
            TraceContext::parse("00-ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ-00f067aa0ba902b7-01")
                .unwrap_err();
        assert!(err.contains("valid hex"));
    }

    #[test]
    fn rejects_wrong_length_trace_id() {
        assert!(TraceContext::parse("00-4bf92f-00f067aa0ba902b7-01").is_err());
    }
}
