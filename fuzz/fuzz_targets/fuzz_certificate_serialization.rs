#![no_main]

use libfuzzer_sys::fuzz_target;
use clap_noun_verb::autonomic::{CertificateId, SchemaHash};

fuzz_target!(|data: &[u8]| {
    // Fuzz CertificateId parsing from arbitrary bytes
    if let Ok(s) = std::str::from_utf8(data) {
        let cert_id = CertificateId::new(s);

        // Verify ID creation is consistent
        let cert_id2 = CertificateId::new(s);
        assert_eq!(cert_id, cert_id2, "CertificateId creation must be deterministic");

        // Test Display trait
        let display = format!("{}", cert_id);
        assert!(!display.is_empty(), "Display should produce output");
    }

    // Fuzz SchemaHash computation
    // SchemaHash should handle arbitrary data without panicking
    if data.len() >= 8 {
        let slice1 = &data[..data.len() / 2];
        let slice2 = &data[data.len() / 2..];

        // Create hashes from arbitrary data
        let _hash1 = SchemaHash::from_bytes(slice1);
        let _hash2 = SchemaHash::from_bytes(slice2);

        // Verify hash determinism
        let h1a = SchemaHash::from_bytes(slice1);
        let h1b = SchemaHash::from_bytes(slice1);
        assert_eq!(h1a, h1b, "Schema hashing must be deterministic");
    }
});
