#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::Address as _, vec, Env, String};

    use crate::{ConfidenceFactor, PetChainContract, PetChainContractClient, RouteConfidence};

    fn make_env() -> Env {
        Env::default()
    }

    fn client(env: &Env) -> PetChainContractClient {
        let id = env.register_contract(None, PetChainContract);
        PetChainContractClient::new(env, &id)
    }

    #[test]
    fn test_fallback_when_no_diagnostics() {
        let env = make_env();
        let c = client(&env);
        let result = c.get_route_confidence(&String::from_str(&env, "best"));
        assert!(result.unavailable);
        assert_eq!(result.score, 0);
        assert_eq!(result.factors.len(), 0);
    }

    #[test]
    fn test_set_and_get_confidence() {
        let env = make_env();
        let c = client(&env);

        let factors = vec![
            &env,
            ConfidenceFactor {
                name: String::from_str(&env, "liquidity_depth"),
                score: 80,
            },
            ConfidenceFactor {
                name: String::from_str(&env, "volatility"),
                score: 60,
            },
            ConfidenceFactor {
                name: String::from_str(&env, "source_freshness"),
                score: 90,
            },
        ];

        let confidence = RouteConfidence {
            score: 77,
            unavailable: false,
            factors: factors.clone(),
        };

        c.set_route_confidence(&String::from_str(&env, "best"), &confidence);
        let result = c.get_route_confidence(&String::from_str(&env, "best"));

        assert!(!result.unavailable);
        assert_eq!(result.score, 77);
        assert_eq!(result.factors.len(), 3);
        assert_eq!(result.factors.get(0).unwrap().score, 80);
        assert_eq!(result.factors.get(1).unwrap().score, 60);
        assert_eq!(result.factors.get(2).unwrap().score, 90);
    }

    #[test]
    fn test_alternative_route_confidence() {
        let env = make_env();
        let c = client(&env);

        let confidence = RouteConfidence {
            score: 45,
            unavailable: false,
            factors: vec![
                &env,
                ConfidenceFactor {
                    name: String::from_str(&env, "liquidity_depth"),
                    score: 40,
                },
            ],
        };

        c.set_route_confidence(&String::from_str(&env, "alt_1"), &confidence);
        let result = c.get_route_confidence(&String::from_str(&env, "alt_1"));

        assert!(!result.unavailable);
        assert_eq!(result.score, 45);
        assert_eq!(result.factors.len(), 1);
    }

    #[test]
    fn test_best_and_alt_routes_independent() {
        let env = make_env();
        let c = client(&env);

        c.set_route_confidence(
            &String::from_str(&env, "best"),
            &RouteConfidence {
                score: 90,
                unavailable: false,
                factors: vec![&env],
            },
        );

        // alt route not set — should return fallback
        let alt = c.get_route_confidence(&String::from_str(&env, "alt_1"));
        assert!(alt.unavailable);

        let best = c.get_route_confidence(&String::from_str(&env, "best"));
        assert!(!best.unavailable);
        assert_eq!(best.score, 90);
    }
}
