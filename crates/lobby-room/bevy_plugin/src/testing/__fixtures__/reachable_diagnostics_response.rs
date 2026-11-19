use freenet_stdlib::client_api::{HostResponse, QueryResponse};

use crate::testing;

#[must_use]
pub fn reachable_diagnostics_response() -> HostResponse {
    HostResponse::QueryResponse(QueryResponse::NodeDiagnostics(testing::node_diagnostics(
        Some("0.5"),
        1,
        false,
    )))
}

#[cfg(test)]
mod tests {
    use freenet_stdlib::client_api::{HostResponse, QueryResponse};

    use super::reachable_diagnostics_response;

    #[test]
    fn test_usage() {
        assert!(matches!(
            reachable_diagnostics_response(),
            HostResponse::QueryResponse(QueryResponse::NodeDiagnostics(diagnostics))
                if diagnostics.node_info.is_some()
        ));
    }
}
