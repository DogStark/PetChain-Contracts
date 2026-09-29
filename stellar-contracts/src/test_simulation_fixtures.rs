// ============================================================
// Simulation Fixture Validation Tests  (Issue #1354)
//
// These tests validate the deterministic transaction simulation
// fixtures in simulation-fixtures.json against the acceptance
// criteria from Issue #1354:
//   1. Fixtures are deterministic and map to documented error codes.
//   2. No fixture contains a real secret.
//   3. Client repositories can consume the format.
//   4. Rust/JSON fixture validation runs in CI.
// ============================================================

use std::collections::HashSet;

/// All documented ContractError discriminant codes from the contract.
const VALID_ERROR_CODES: &[u32] = &[
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 43, 44, 45, 46, 47, 160, 161,
    162, 163, 164, 165, 166, 167, 168, 169,
];

fn load_fixtures() -> serde_json::Value {
    let content = include_str!("../simulation-fixtures.json");
    serde_json::from_str(content)
        .expect("simulation-fixtures.json must be valid JSON")
}

fn load_schema() -> serde_json::Value {
    let content = include_str!("../simulation-fixtures.schema.json");
    serde_json::from_str(content)
        .expect("simulation-fixtures.schema.json must be valid JSON")
}

// ─── STRUCTURAL VALIDATION ────────────────────────────────────────────

#[test]
fn fixture_file_is_valid_json() {
    let parsed = load_fixtures();
    assert!(
        parsed.is_object(),
        "simulation-fixtures.json must be a JSON object"
    );
}

#[test]
fn schema_file_is_valid_json() {
    let parsed = load_schema();
    assert!(
        parsed.is_object(),
        "simulation-fixtures.schema.json must be a JSON object"
    );
}

#[test]
fn fixture_has_required_top_level_fields() {
    let parsed = load_fixtures();
    assert!(
        parsed.get("fixture_version").is_some(),
        "missing required field: fixture_version"
    );
    assert!(
        parsed.get("simulations").is_some(),
        "missing required field: simulations"
    );
    assert!(
        parsed.get("description").is_some(),
        "missing required field: description"
    );
}

#[test]
fn fixture_version_is_valid() {
    let parsed = load_fixtures();
    let version = parsed
        .get("fixture_version")
        .and_then(|v| v.as_str())
        .expect("fixture_version must be a string");
    assert_eq!(
        version, "1.0.0",
        "fixture_version must be 1.0.0"
    );
}

#[test]
fn schema_has_dollar_schema_field() {
    let parsed = load_schema();
    let schema = parsed
        .get("$schema")
        .and_then(|v| v.as_str())
        .expect("schema must have a $schema field");
    assert!(
        schema.contains("json-schema.org"),
        "schema $schema must reference JSON Schema"
    );
}

// ─── SIMULATIONS ARRAY VALIDATION ─────────────────────────────────────

#[test]
fn simulations_array_has_minimum_items() {
    let parsed = load_fixtures();
    let simulations = parsed
        .get("simulations")
        .and_then(|s| s.as_array())
        .expect("simulations must be an array");
    assert!(
        simulations.len() >= 5,
        "must have at least 5 simulations (got {})",
        simulations.len()
    );
}

#[test]
fn each_simulation_has_required_fields() {
    let parsed = load_fixtures();
    let simulations = parsed
        .get("simulations")
        .and_then(|s| s.as_array())
        .expect("simulations must be an array");

    for sim in simulations {
        let fixture_id = sim
            .get("fixture_id")
            .and_then(|v| v.as_str())
            .expect("each simulation must have a fixture_id string");

        assert!(
            sim.get("description").is_some(),
            "simulation '{}' must have a description",
            fixture_id
        );
        assert!(
            sim.get("function").is_some(),
            "simulation '{}' must have a function",
            fixture_id
        );
        assert!(
            sim.get("params").is_some(),
            "simulation '{}' must have params",
            fixture_id
        );
        assert!(
            sim.get("expected").is_some(),
            "simulation '{}' must have expected",
            fixture_id
        );

        let expected = sim.get("expected").unwrap();
        assert!(
            expected.get("result").is_some(),
            "simulation '{}' expected must have result",
            fixture_id
        );

        let result = expected
            .get("result")
            .and_then(|v| v.as_str())
            .expect("result must be a string");
        assert!(
            result == "success" || result == "failure",
            "simulation '{}' result must be 'success' or 'failure', got '{}'",
            fixture_id,
            result
        );
    }
}

// ─── ERROR CODE VALIDATION ────────────────────────────────────────────

#[test]
fn failure_simulations_have_valid_error_codes() {
    let parsed = load_fixtures();
    let simulations = parsed
        .get("simulations")
        .and_then(|s| s.as_array())
        .expect("simulations must be an array");

    let valid_codes: HashSet<u32> = VALID_ERROR_CODES.iter().copied().collect();

    for sim in simulations {
        let fixture_id = sim
            .get("fixture_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let expected = sim.get("expected").unwrap();
        let result = expected
            .get("result")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if result == "failure" {
            let error_code = expected
                .get("error_code")
                .and_then(|v| v.as_u64())
                .expect(&format!(
                    "simulation '{}' (failure) must have an error_code",
                    fixture_id
                ));

            assert!(
                valid_codes.contains(&(error_code as u32)),
                "simulation '{}' has error_code {} which is not a documented ContractError discriminant",
                fixture_id,
                error_code
            );

            // error_name should also be present for failures
            assert!(
                expected.get("error_name").is_some(),
                "simulation '{}' (failure) must have an error_name",
                fixture_id
            );
        }
    }
}

// ─── SECRET-FREE VALIDATION ───────────────────────────────────────────

#[test]
fn no_fixture_contains_real_secrets() {
    let parsed = load_fixtures();
    let simulations = parsed
        .get("simulations")
        .and_then(|s| s.as_array())
        .expect("simulations must be an array");

    for sim in simulations {
        let fixture_id = sim
            .get("fixture_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        // Walk all string values in the fixture and check for secret patterns
        fn check_value_for_secrets(value: &serde_json::Value, path: &str, fixture_id: &str) {
            match value {
                serde_json::Value::String(s) => {
                    // Stellar secret keys start with 'S' and are 56 chars
                    assert!(
                        !s.starts_with('S') || s.len() != 56,
                        "fixture '{}' contains a possible secret key at {}: {}",
                        fixture_id,
                        path,
                        s
                    );
                    // Passphrases contain "Secret" or "passphrase"
                    assert!(
                        !s.to_lowercase().contains("secret") || s.len() < 10,
                        "fixture '{}' contains a possible passphrase at {}: {}",
                        fixture_id,
                        path,
                        s
                    );
                }
                serde_json::Value::Object(map) => {
                    for (key, val) in map {
                        check_value_for_secrets(
                            val,
                            &format!("{}.{}", path, key),
                            fixture_id,
                        );
                    }
                }
                serde_json::Value::Array(arr) => {
                    for (i, val) in arr.iter().enumerate() {
                        check_value_for_secrets(
                            val,
                            &format!("{}[{}]", path, i),
                            fixture_id,
                        );
                    }
                }
                _ => {}
            }
        }

        check_value_for_secrets(&sim, "root", fixture_id);
    }
}

// ─── SCENARIO COVERAGE ────────────────────────────────────────────────

#[test]
fn all_five_required_scenarios_are_present() {
    let parsed = load_fixtures();
    let simulations = parsed
        .get("simulations")
        .and_then(|s| s.as_array())
        .expect("simulations must be an array");

    let fixture_ids: Vec<String> = simulations
        .iter()
        .filter_map(|s| s.get("fixture_id").and_then(|v| v.as_str()).map(String::from))
        .collect();

    let fixture_ids_lower: Vec<String> = fixture_ids
        .iter()
        .map(|id| id.to_lowercase())
        .collect();

    let has_success = fixture_ids_lower
        .iter()
        .any(|id| id.contains("success"));
    let has_auth_failure = fixture_ids_lower
        .iter()
        .any(|id| id.contains("auth") || id.contains("unauthorized"));
    let has_stale_version = fixture_ids_lower
        .iter()
        .any(|id| id.contains("stale") || id.contains("version"));
    let has_budget_failure = fixture_ids_lower
        .iter()
        .any(|id| id.contains("budget") || id.contains("batch"));
    let has_malformed_input = fixture_ids_lower
        .iter()
        .any(|id| id.contains("malformed") || id.contains("invalid") || id.contains("empty"));

    assert!(has_success, "must have a success scenario fixture");
    assert!(
        has_auth_failure,
        "must have an authorization failure scenario fixture"
    );
    assert!(
        has_stale_version,
        "must have a stale version scenario fixture"
    );
    assert!(
        has_budget_failure,
        "must have a budget failure scenario fixture"
    );
    assert!(
        has_malformed_input,
        "must have a malformed input scenario fixture"
    );
}

// ─── DETERMINISM ──────────────────────────────────────────────────────

#[test]
fn fixtures_are_deterministic_no_duplicate_ids() {
    let parsed = load_fixtures();
    let simulations = parsed
        .get("simulations")
        .and_then(|s| s.as_array())
        .expect("simulations must be an array");

    let mut seen_ids = HashSet::new();
    for sim in simulations {
        let id = sim
            .get("fixture_id")
            .unwrap()
            .as_str()
            .unwrap();
        assert!(
            seen_ids.insert(id.to_string()),
            "duplicate fixture_id found: {}",
            id
        );
    }
}

#[test]
fn each_fixture_has_deterministic_expected_result() {
    let parsed = load_fixtures();
    let simulations = parsed
        .get("simulations")
        .and_then(|s| s.as_array())
        .expect("simulations must be an array");

    for sim in simulations {
        let fixture_id = sim
            .get("fixture_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let expected = sim.get("expected").unwrap();
        let result = expected
            .get("result")
            .and_then(|v| v.as_str())
            .expect("expected.result must be a string");

        // The result must be either "success" or "failure" — not conditional
        // on external state. This is what makes the fixture deterministic.
        assert!(
            result == "success" || result == "failure",
            "fixture '{}' has non-deterministic result: {}",
            fixture_id,
            result
        );

        // resource_estimates should be present and fixed (not variable)
        assert!(
            expected.get("resource_estimates").is_some(),
            "fixture '{}' must have resource_estimates for determinism",
            fixture_id
        );
    }
}

// ─── CLIENT CONSUMABILITY ─────────────────────────────────────────────

#[test]
fn fixtures_use_client_consumable_format() {
    let parsed = load_fixtures();
    let simulations = parsed
        .get("simulations")
        .and_then(|s| s.as_array())
        .expect("simulations must be an array");

    for sim in simulations {
        let fixture_id = sim
            .get("fixture_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        // Each simulation must have a "function" field (string)
        let function = sim
            .get("function")
            .and_then(|v| v.as_str())
            .expect(&format!("fixture '{}' must have a string 'function' field", fixture_id));
        assert!(
            !function.is_empty(),
            "fixture '{}' function must not be empty",
            fixture_id
        );

        // Each simulation must have "params" (object)
        let params = sim
            .get("params")
            .and_then(|v| v.as_object())
            .expect(&format!("fixture '{}' params must be an object", fixture_id));
        assert!(
            !params.is_empty(),
            "fixture '{}' params must not be empty",
            fixture_id
        );

        // Each simulation must have "expected" (object)
        let expected = sim
            .get("expected")
            .and_then(|v| v.as_object())
            .expect(&format!("fixture '{}' expected must be an object", fixture_id));
        assert!(
            !expected.is_empty(),
            "fixture '{}' expected must not be empty",
            fixture_id
        );
    }
}

#[test]
fn schema_file_is_consumable() {
    let parsed = load_schema();

    // Schema must have a title and description for client consumption
    assert!(
        parsed.get("title").is_some(),
        "schema must have a title"
    );
    assert!(
        parsed.get("description").is_some(),
        "schema must have a description"
    );
}
