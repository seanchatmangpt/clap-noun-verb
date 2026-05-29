use clap_noun_verb_utils::number_parsing::{
    decimal_range, maybe_hex, maybe_hex_range, parse_bytes, parse_duration, parse_percentage,
};
use std::time::Duration;

#[test]
fn test_decimal_range() -> Result<(), String> {
    let parse = decimal_range(10, 20);
    
    let v1 = parse("15")?;
    assert_eq!(v1, 15);
    
    let v2 = parse("10")?;
    assert_eq!(v2, 10);
    
    let v3 = parse("20")?;
    assert_eq!(v3, 20);
    
    assert!(parse("5").is_err());
    assert!(parse("25").is_err());
    assert!(parse("abc").is_err());
    
    Ok(())
}

#[test]
fn test_maybe_hex() -> Result<(), String> {
    let v1: u32 = maybe_hex("10")?;
    assert_eq!(v1, 10);
    
    let v2: u32 = maybe_hex("0x10")?;
    assert_eq!(v2, 16);
    
    let v3: u32 = maybe_hex("0X1A")?;
    assert_eq!(v3, 26);
    
    assert!(maybe_hex::<u32>("abc").is_err());
    assert!(maybe_hex::<u32>("-1").is_err()); // u32 can't be negative
    
    Ok(())
}

#[test]
fn test_maybe_hex_range() -> Result<(), String> {
    let parse = maybe_hex_range(10u32, 20u32);
    
    let v1 = parse("12")?;
    assert_eq!(v1, 12);
    
    let v2 = parse("0x0C")?; // 12 in hex
    assert_eq!(v2, 12);
    
    let v3 = parse("20")?;
    assert_eq!(v3, 20);
    
    assert!(parse("9").is_err());
    assert!(parse("0x15").is_err()); // 21 in hex, out of range
    
    Ok(())
}

#[test]
fn test_parse_percentage() -> Result<(), String> {
    let p1 = parse_percentage("50%")?;
    assert!((p1 - 0.5).abs() < 1e-6);
    
    let p2 = parse_percentage("12.5%")?;
    assert!((p2 - 0.125).abs() < 1e-6);
    
    let p3 = parse_percentage("0%")?;
    assert!((p3 - 0.0).abs() < 1e-6);
    
    let p4 = parse_percentage("100%")?;
    assert!((p4 - 1.0).abs() < 1e-6);
    
    assert!(parse_percentage("101%").is_err());
    assert!(parse_percentage("-5%").is_err());
    assert!(parse_percentage("50").is_err()); // missing %
    
    Ok(())
}

#[test]
fn test_parse_bytes() -> Result<(), String> {
    assert_eq!(parse_bytes("5b")?, 5);
    assert_eq!(parse_bytes("10kb")?, 10240);
    assert_eq!(parse_bytes("5mb")?, 5242880);
    assert_eq!(parse_bytes("2g")?, 2147483648);
    assert_eq!(parse_bytes("2 gb")?, 2147483648);
    
    assert!(parse_bytes("abc").is_err());
    assert!(parse_bytes("10xb").is_err());
    
    Ok(())
}

#[test]
fn test_parse_duration() -> Result<(), String> {
    assert_eq!(parse_duration("30s")?, Duration::from_secs(30));
    assert_eq!(parse_duration("5m")?, Duration::from_secs(300));
    assert_eq!(parse_duration("1h 15m")?, Duration::from_secs(4500));
    assert_eq!(parse_duration("1d 2h")?, Duration::from_secs(93600));
    
    assert!(parse_duration("abc").is_err());
    assert!(parse_duration("30").is_err()); // missing unit
    
    Ok(())
}
