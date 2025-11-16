#![no_main]

use libfuzzer_sys::fuzz_target;
use clap_noun_verb::autonomic::CapabilityId;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, handling invalid UTF-8 gracefully
    if let Ok(s) = std::str::from_utf8(data) {
        // Test CapabilityId creation from arbitrary input
        let _id = CapabilityId::from_path(s);

        // Test versioned capability parsing
        if let Some((path, version)) = s.split_once('@') {
            let _versioned = CapabilityId::from_path_versioned(path, version);
        }

        // Test that parsing is deterministic
        let id1 = CapabilityId::from_path(s);
        let id2 = CapabilityId::from_path(s);
        assert_eq!(id1.as_str(), id2.as_str(), "Parsing must be deterministic");
    }
});
