use clap_noun_verb_utils::number_parsing::{
    decimal_range, maybe_hex_range, parse_bytes, parse_duration, parse_percentage,
};
use clap_noun_verb_utils::adapters::LayeredConfigAdapter;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

static CHALLENGE_ENV_MUTEX: Mutex<()> = Mutex::new(());

struct TempCleanup(PathBuf);
impl Drop for TempCleanup {
    fn drop(&mut self) {
        if self.0.exists() {
            let _ = fs::remove_file(&self.0);
        }
    }
}

// ---------------------------------------------------------
// 1. Number Parsing Verification Tests
// ---------------------------------------------------------

#[test]
fn test_number_parsing_overflow_empty_invalid_limits() {
    // A. decimal_range boundary & overflow checks
    let parse_u8 = decimal_range(0u8, 255u8);
    // Overflow inputs
    assert!(parse_u8("256").is_err());
    assert!(parse_u8("1000000000000").is_err());
    assert!(parse_u8("-1").is_err());
    // Empty inputs
    assert!(parse_u8("").is_err());
    assert!(parse_u8("   ").is_err());
    // Invalid range configuration: min > max
    let parse_invalid_dec = decimal_range(100, 0);
    assert!(parse_invalid_dec("50").is_err());

    // B. maybe_hex_range boundary & overflow checks
    let parse_hex_u16 = maybe_hex_range(10u16, 1000u16);
    // Overflow inputs (dec and hex)
    assert!(parse_hex_u16("1001").is_err());
    assert!(parse_hex_u16("0x3EA").is_err()); // 1002 in hex, out of range
    assert!(parse_hex_u16("65536").is_err()); // u16 overflow
    assert!(parse_hex_u16("0x10000").is_err()); // u16 overflow in hex
    assert!(parse_hex_u16("18446744073709551616").is_err()); // huge overflow
    // Empty inputs
    assert!(parse_hex_u16("").is_err());
    assert!(parse_hex_u16("  ").is_err());
    // Invalid range configuration: min > max
    let parse_invalid_hex = maybe_hex_range(50u32, 10u32);
    assert!(parse_invalid_hex("20").is_err());

    // C. parse_percentage checks
    // Overflow/underflow bounds
    assert!(parse_percentage("-0.0001%").is_err());
    assert!(parse_percentage("100.0001%").is_err());
    assert!(parse_percentage("1000%").is_err());
    // Empty inputs
    assert!(parse_percentage("").is_err());
    assert!(parse_percentage("%").is_err());
    assert!(parse_percentage("   %").is_err());
    // Special values: NaN / inf
    assert!(parse_percentage("NaN%").is_err());
    assert!(parse_percentage("inf%").is_err());
    assert!(parse_percentage("-inf%").is_err());

    // D. parse_bytes checks
    // Overflow inputs
    assert!(parse_bytes("18446744073709551616").is_err()); // u64::MAX + 1
    assert!(parse_bytes("18446744073709551615kb").is_err()); // Multiplier overflow
    assert!(parse_bytes("18446744073709551615t").is_err()); // Multiplier overflow
    assert!(parse_bytes("99999999999999999999999999999g").is_err());
    // Empty inputs
    assert!(parse_bytes("").is_err());
    assert!(parse_bytes("   ").is_err());

    // E. parse_duration checks
    // Overflow inputs
    assert!(parse_duration("18446744073709551615d").is_err());
    assert!(parse_duration("18446744073709551615h").is_err());
    assert!(parse_duration("18446744073709551615m").is_err());
    // Empty inputs
    assert!(parse_duration("").is_err());
    assert!(parse_duration("  \n\t ").is_err());
    // Invalid units and formats
    assert!(parse_duration("10").is_err());
    assert!(parse_duration("s").is_err());
    assert!(parse_duration("1.5s").is_err());
    assert!(parse_duration("1s 2x").is_err());
}

// ---------------------------------------------------------
// 2. Configuration Adapter Merging Verification Tests
// ---------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
struct SubConfig {
    pub name: String,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
struct DeepConfig {
    pub level: u32,
    pub sub: SubConfig,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
struct AppConfig {
    pub host: String,
    pub port: u16,
    pub debug: bool,
    pub deep: DeepConfig,
}

#[test]
fn test_configuration_adapter_nested_merges() {
    let _lock = CHALLENGE_ENV_MUTEX.lock().unwrap();

    // Clean env variables that we will use
    let env_vars = [
        "CFG_PORT",
        "CFG_DEBUG",
        "CFG_DEEP__LEVEL",
        "CFG_DEEP__SUB__NAME",
        "CFG_DEEP__SUB__ACTIVE",
    ];
    for var in &env_vars {
        std::env::remove_var(var);
    }

    // A. Default Value Check
    // We expect AppConfig to serialize to default values
    let adapter: LayeredConfigAdapter<AppConfig> = LayeredConfigAdapter::new(None, None);
    let cmd = clap::Command::new("app");
    let matches = cmd.try_get_matches_from(vec!["app"]).unwrap();
    let resolved_default = adapter.resolve(&matches).unwrap();
    assert_eq!(resolved_default, AppConfig::default());
    assert_eq!(resolved_default.host, "");
    assert_eq!(resolved_default.port, 0);
    assert!(!resolved_default.debug);
    assert_eq!(resolved_default.deep.level, 0);
    assert_eq!(resolved_default.deep.sub.name, "");
    assert!(!resolved_default.deep.sub.active);

    // B. Merge with Config File (Deeply nested)
    let temp_json = TempCleanup(std::env::temp_dir().join(format!("cnv_challenge_cfg_{}.json", std::process::id())));
    let config_json = r#"{
        "host": "config-host",
        "port": 8080,
        "deep": {
            "level": 1,
            "sub": {
                "name": "config-sub",
                "active": true
            }
        }
    }"#;
    fs::write(&temp_json.0, config_json).unwrap();

    let adapter_file: LayeredConfigAdapter<AppConfig> = LayeredConfigAdapter::new(Some(temp_json.0.clone()), None);
    let resolved_file = adapter_file.resolve(&matches).unwrap();
    assert_eq!(resolved_file.host, "config-host");
    assert_eq!(resolved_file.port, 8080);
    assert_eq!(resolved_file.deep.level, 1);
    assert_eq!(resolved_file.deep.sub.name, "config-sub");
    assert!(resolved_file.deep.sub.active);

    // C. Merge with environment variables (using double-underscore delimiter for nested keys)
    std::env::set_var("CFG_PORT", "9090"); // flat override
    std::env::set_var("CFG_DEEP__LEVEL", "2"); // nested level 1 override
    std::env::set_var("CFG_DEEP__SUB__NAME", "env-sub"); // nested level 2 override

    let adapter_env: LayeredConfigAdapter<AppConfig> = LayeredConfigAdapter::new(Some(temp_json.0.clone()), Some("CFG_".to_string()));
    let resolved_env = adapter_env.resolve(&matches).unwrap();

    // port: config file (8080) -> env override (9090) => 9090
    assert_eq!(resolved_env.port, 9090);
    // deep.level: config file (1) -> env override (2) => 2
    assert_eq!(resolved_env.deep.level, 2);
    // deep.sub.name: config file ("config-sub") -> env override ("env-sub") => "env-sub"
    assert_eq!(resolved_env.deep.sub.name, "env-sub");
    // deep.sub.active: config file (true) -> env (not set) => true (retains file value)
    assert!(resolved_env.deep.sub.active);
    // host: config file ("config-host") -> env (not set) => "config-host"
    assert_eq!(resolved_env.host, "config-host");

    // D. CLI overrides and default value checks
    let cmd_with_args = clap::Command::new("app")
        .arg(
            clap::Arg::new("host")
                .long("host")
                .action(clap::ArgAction::Set)
                .default_value("default-cli-host"), // tests default check
        )
        .arg(
            clap::Arg::new("deep.level")
                .long("deep-level")
                .action(clap::ArgAction::Set),
        )
        .arg(
            clap::Arg::new("deep.sub.active")
                .long("deep-sub-active")
                .action(clap::ArgAction::SetTrue),
        );

    // Run 1: CLI doesn't override "host" (it uses default), overrides "deep.level" and "deep.sub.active"
    let matches_cli = cmd_with_args
        .clone()
        .try_get_matches_from(vec!["app", "--deep-level", "3", "--deep-sub-active"])
        .unwrap();

    let resolved_cli = adapter_env.resolve(&matches_cli).unwrap();

    // host: default CLI value ("default-cli-host") should NOT override config file value ("config-host")
    assert_eq!(resolved_cli.host, "config-host");
    // deep.level: cli override ("3") should override env (2) and config file (1) => 3
    assert_eq!(resolved_cli.deep.level, 3);
    // deep.sub.active: cli flag true should override config file (true) => true
    assert!(resolved_cli.deep.sub.active);

    // Run 2: CLI explicitly overrides "host"
    let matches_cli_explicit = cmd_with_args
        .try_get_matches_from(vec!["app", "--host", "cli-explicit-host"])
        .unwrap();

    let resolved_cli_explicit = adapter_env.resolve(&matches_cli_explicit).unwrap();
    // host: explicit cli value should override config file => "cli-explicit-host"
    assert_eq!(resolved_cli_explicit.host, "cli-explicit-host");

    // Clean up env
    for var in &env_vars {
        std::env::remove_var(var);
    }
}
