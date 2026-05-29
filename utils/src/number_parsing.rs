use std::str::FromStr;
use std::time::Duration;

/// Parses decimal integers restricted to a closed interval [min, max].
/// Wraps `clap_num::number_range`.
///
/// # Examples
///
/// ```
/// use clap_noun_verb_utils::number_parsing::decimal_range;
///
/// # fn main() -> Result<(), String> {
/// let parser = decimal_range(10, 20);
/// let val = parser("15")?;
/// assert_eq!(val, 15);
/// assert!(parser("5").is_err());
/// assert!(parser("25").is_err());
/// assert!(parser("abc").is_err());
/// # Ok(())
/// # }
/// ```
pub fn decimal_range<T>(min: T, max: T) -> impl Fn(&str) -> Result<T, String>
where
    T: FromStr + Copy + Ord + std::fmt::Display,
    <T as FromStr>::Err: std::fmt::Display,
{
    move |s| {
        if min > max {
            return Err(format!("Invalid range configuration: min ({}) > max ({})", min, max));
        }
        clap_num::number_range(s, min, max)
    }
}

/// Parses an integer that can be either decimal or hexadecimal (prefixed with 0x or 0X).
/// Wraps `clap_num::maybe_hex`.
///
/// # Examples
///
/// ```
/// use clap_noun_verb_utils::number_parsing::maybe_hex;
///
/// # fn main() -> Result<(), String> {
/// let val_dec: u32 = maybe_hex("10")?;
/// assert_eq!(val_dec, 10);
///
/// let val_hex: u32 = maybe_hex("0x0A")?;
/// assert_eq!(val_hex, 10);
///
/// assert!(maybe_hex::<u32>("abc").is_err());
/// # Ok(())
/// # }
/// ```
pub fn maybe_hex<T>(s: &str) -> Result<T, String>
where
    T: num_traits::Num + num_traits::Unsigned,
    <T as num_traits::Num>::FromStrRadixErr: std::fmt::Display,
{
    clap_num::maybe_hex(s)
}

/// Parses an integer (decimal/hex) restricted to a closed interval [min, max].
/// Wraps `clap_num::maybe_hex_range`.
///
/// # Examples
///
/// ```
/// use clap_noun_verb_utils::number_parsing::maybe_hex_range;
///
/// # fn main() -> Result<(), String> {
/// let parser = maybe_hex_range(10u32, 20u32);
/// let val_dec = parser("15")?;
/// assert_eq!(val_dec, 15);
///
/// let val_hex = parser("0x0F")?;
/// assert_eq!(val_hex, 15);
///
/// assert!(parser("5").is_err());
/// assert!(parser("0x15").is_err());
/// # Ok(())
/// # }
/// ```
pub fn maybe_hex_range<T>(min: T, max: T) -> impl Fn(&str) -> Result<T, String>
where
    T: num_traits::Num + num_traits::Unsigned + FromStr + Ord + Copy + std::fmt::Display,
    <T as num_traits::Num>::FromStrRadixErr: std::fmt::Display,
    <T as FromStr>::Err: std::fmt::Display,
{
    move |s| {
        if min > max {
            return Err(format!("Invalid range configuration: min ({}) > max ({})", min, max));
        }
        clap_num::maybe_hex_range(s, min, max)
    }
}

/// Custom format: Parses human-readable percentage strings (e.g., "50%", "12.5%") to f64 values (0.5, 0.125).
///
/// # Examples
///
/// ```
/// use clap_noun_verb_utils::number_parsing::parse_percentage;
///
/// # fn main() -> Result<(), String> {
/// let val = parse_percentage("50%")?;
/// assert!((val - 0.5).abs() < f64::EPSILON);
///
/// let val2 = parse_percentage("12.5%")?;
/// assert!((val2 - 0.125).abs() < f64::EPSILON);
///
/// assert!(parse_percentage("150%").is_err());
/// assert!(parse_percentage("50").is_err());
/// # Ok(())
/// # }
/// ```
pub fn parse_percentage(s: &str) -> Result<f64, String> {
    if !s.ends_with('%') {
        return Err("Percentage must end with '%'".to_string());
    }
    let val_str = &s[..s.len() - 1];
    let val = val_str.parse::<f64>().map_err(|e| format!("Invalid percentage: {}", e))?;
    if !(0.0..=100.0).contains(&val) {
        return Err("Percentage must be between 0% and 100%".to_string());
    }
    Ok(val / 100.0)
}

/// Custom format: Parses human-readable byte sizes (e.g., "10kb", "5MB", "2g") to u64 bytes.
///
/// # Examples
///
/// ```
/// use clap_noun_verb_utils::number_parsing::parse_bytes;
///
/// # fn main() -> Result<(), String> {
/// let b = parse_bytes("10kb")?;
/// assert_eq!(b, 10240);
///
/// let b2 = parse_bytes("1m")?;
/// assert_eq!(b2, 1048576);
///
/// assert!(parse_bytes("invalid").is_err());
/// # Ok(())
/// # }
/// ```
pub fn parse_bytes(s: &str) -> Result<u64, String> {
    let s_lower = s.to_lowercase();
    let alphabetic_pos = s_lower.find(|c: char| c.is_alphabetic()).unwrap_or(s_lower.len());
    let (num_part, unit_part) = s_lower.split_at(alphabetic_pos);
    let number =
        num_part.trim().parse::<u64>().map_err(|e| format!("Invalid byte number: {}", e))?;

    let multiplier = match unit_part.trim() {
        "" | "b" => 1,
        "k" | "kb" => 1024,
        "m" | "mb" => 1024 * 1024,
        "g" | "gb" => 1024 * 1024 * 1024,
        "t" | "tb" => 1024 * 1024 * 1024 * 1024,
        unknown => return Err(format!("Unknown byte unit: {}", unknown)),
    };

    number.checked_mul(multiplier).ok_or_else(|| "Byte size overflow".to_string())
}

/// Custom format: Parses duration strings (e.g., "30s", "1h 15m") to std::time::Duration.
///
/// # Examples
///
/// ```
/// use clap_noun_verb_utils::number_parsing::parse_duration;
/// use std::time::Duration;
///
/// # fn main() -> Result<(), String> {
/// let d = parse_duration("30s")?;
/// assert_eq!(d, Duration::from_secs(30));
///
/// let d2 = parse_duration("1h 15m")?;
/// assert_eq!(d2, Duration::from_secs(4500));
///
/// assert!(parse_duration("").is_err());
/// assert!(parse_duration("30").is_err());
/// # Ok(())
/// # }
/// ```
pub fn parse_duration(s: &str) -> Result<Duration, String> {
    if s.trim().is_empty() {
        return Err("Duration cannot be empty".to_string());
    }
    let mut total_secs = 0u64;
    let words = s.split_whitespace();
    for word in words {
        let pos = word
            .find(|c: char| c.is_alphabetic())
            .ok_or_else(|| "Missing unit in duration segment".to_string())?;
        let (num_part, unit_part) = word.split_at(pos);
        let val = num_part.parse::<u64>().map_err(|e| format!("Invalid duration value: {}", e))?;
        let secs = match unit_part {
            "s" | "sec" | "secs" => Some(val),
            "m" | "min" | "mins" => val.checked_mul(60),
            "h" | "hour" | "hours" => val.checked_mul(3600),
            "d" | "day" | "days" => val.checked_mul(86400),
            unknown => return Err(format!("Unknown duration unit: {}", unknown)),
        }
        .ok_or_else(|| "Duration overflow".to_string())?;
        total_secs = total_secs.checked_add(secs).ok_or_else(|| "Duration overflow".to_string())?;
    }
    Ok(Duration::from_secs(total_secs))
}
