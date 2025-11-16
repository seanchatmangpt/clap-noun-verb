#![no_main]

use libfuzzer_sys::fuzz_target;
use clap_noun_verb::autonomic::ZeroCopyParser;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let parser = ZeroCopyParser::new(s);

        // Fuzz argument extraction
        for i in 0..10 {
            let _ = parser.arg(i);
        }

        // Fuzz named argument extraction
        if let Some(key) = s.split_whitespace().next() {
            let _ = parser.named(key);
        }

        // Verify substring property: all returned slices must be substrings
        for i in 0..parser.arg_count() {
            if let Some(arg) = parser.arg(i) {
                assert!(
                    s.contains(arg),
                    "Zero-copy parser must return substrings of original input"
                );
            }
        }
    }
});
