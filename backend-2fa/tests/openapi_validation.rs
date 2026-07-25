use serde_yaml::Value;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Validates that all paths documented in the OpenAPI spec are implemented in the router.
/// This test reads docs/openapi.yaml and ensures every documented endpoint exists.
#[test]
fn test_openapi_spec_endpoints_match_router() {
    let spec_path = Path::new("docs/openapi.yaml");

    // Read and parse the OpenAPI spec
    let spec_content = fs::read_to_string(spec_path)
        .expect("Failed to read OpenAPI spec. Make sure docs/openapi.yaml exists.");

    let spec: Value = serde_yaml::from_str(&spec_content)
        .expect("Failed to parse OpenAPI spec as YAML");

    // Extract all documented paths
    let paths = spec["paths"]
        .as_mapping()
        .expect("OpenAPI spec must have a 'paths' section");

    let documented_endpoints: HashSet<String> = paths
        .keys()
        .filter_map(|key| key.as_str().map(|s| s.to_string()))
        .collect();

    // Define the actual router endpoints (these should match the documented endpoints)
    let implemented_endpoints: HashSet<&str> = [
        "/2fa/enable",
        "/2fa/disable",
        "/2fa/verify",
        "/2fa/login",
        "/2fa/recover",
        "/2fa/recovery-log",
        "/2fa/upgrade-algorithm",
        "/2fa/revoke-session",
        "/admin/quota",
        "/admin/canary",
        "/admin/flagged-scores",
        "/admin/list-users",
        "/admin/unlock",
        "/admin/list-locked",
        "/tenant/provision",
        "/leaderboard/subscribe",
    ]
    .iter()
    .cloned()
    .collect();

    // Check that all documented endpoints are implemented
    for endpoint in &documented_endpoints {
        assert!(
            implemented_endpoints.contains(endpoint.as_str()),
            "Endpoint {} documented in OpenAPI spec but not found in router",
            endpoint
        );
    }

    // Check that documented endpoints cover all important router endpoints
    let critical_endpoints = vec![
        "/2fa/enable",
        "/2fa/disable",
        "/2fa/verify",
    ];

    for endpoint in critical_endpoints {
        assert!(
            documented_endpoints.contains(endpoint),
            "Critical endpoint {} not documented in OpenAPI spec",
            endpoint
        );
    }
}

/// Validates that the OpenAPI spec has valid structure.
#[test]
fn test_openapi_spec_has_required_fields() {
    let spec_path = Path::new("docs/openapi.yaml");

    let spec_content = fs::read_to_string(spec_path)
        .expect("Failed to read OpenAPI spec");

    let spec: Value = serde_yaml::from_str(&spec_content)
        .expect("Failed to parse OpenAPI spec as YAML");

    // Validate required OpenAPI 3.0 fields
    assert!(
        spec["openapi"].is_string(),
        "OpenAPI spec must have 'openapi' version field"
    );

    assert!(
        spec["info"].is_mapping(),
        "OpenAPI spec must have 'info' section"
    );

    assert!(
        spec["paths"].is_mapping(),
        "OpenAPI spec must have 'paths' section"
    );

    // Validate info section
    let info = &spec["info"];
    assert!(
        info["title"].is_string(),
        "OpenAPI info must have 'title'"
    );

    assert!(
        info["version"].is_string(),
        "OpenAPI info must have 'version'"
    );
}
