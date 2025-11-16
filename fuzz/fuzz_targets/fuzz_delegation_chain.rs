#![no_main]

use libfuzzer_sys::fuzz_target;
use clap_noun_verb::autonomic::{
    AgentIdentity, CapabilityConstraint, DelegationToken, Principal, TenantIdentity,
};

fuzz_target!(|data: &[u8]| {
    // Only process reasonably sized inputs to avoid OOM
    if data.len() > 1024 {
        return;
    }

    if let Ok(s) = std::str::from_utf8(data) {
        // Fuzz Principal creation
        let agent = AgentIdentity::anonymous();
        let tenant = TenantIdentity::default_tenant();
        let principal = Principal::new(agent, tenant);

        // Fuzz CapabilityConstraint with arbitrary data
        let parts: Vec<&str> = s.split(',').collect();
        if !parts.is_empty() {
            let constraint = CapabilityConstraint::new(
                parts.iter().map(|p| p.trim().to_string()).collect(),
                None,
                None,
            );

            // Test constraint intersection is commutative
            let c1 = constraint.clone();
            let c2 = CapabilityConstraint::new(vec![], None, None);

            let a_intersect_b = c1.intersect(&c2);
            let b_intersect_a = c2.intersect(&c1);

            assert_eq!(
                a_intersect_b.allowed_capabilities,
                b_intersect_a.allowed_capabilities,
                "Constraint intersection must be commutative"
            );

            // Fuzz token creation
            let _token = DelegationToken::new(
                principal.clone(),
                principal,
                constraint,
                std::time::Duration::from_secs(3600),
            );
        }
    }
});
