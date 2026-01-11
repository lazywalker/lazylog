use serde::{Deserialize, Deserializer, Serialize, de};
#[cfg(feature = "time")]
use time::OffsetDateTime;

/// Parse a size string with optional units (K/M/G, case-insensitive), defaulting to KB if no unit.
fn parse_size(s: &str) -> Result<u64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty size string".to_string());
    }

    let (num_str, unit) = if s.chars().last().unwrap().is_alphabetic() {
        let len = s.len();
        let num_part = &s[..len - 1];
        let unit_char = s.chars().last().unwrap().to_ascii_uppercase();
        (num_part, unit_char)
    } else {
        (s, 'K') // Default to KB
    };

    let num: u64 = num_str
        .parse()
        .map_err(|_| format!("invalid number: {}", num_str))?;

    let multiplier = match unit {
        'K' => 1024,
        'M' => 1024 * 1024,
        'G' => 1024 * 1024 * 1024,
        _ => return Err(format!("invalid unit: {}, supported: K/M/G", unit)),
    };

    num.checked_mul(multiplier)
        .ok_or_else(|| "size too large".to_string())
}

/// Size value that can be a number or string with units.
#[derive(Deserialize)]
#[serde(untagged)]
enum SizeValue {
    Number(u64),
    String(String),
}

impl SizeValue {
    fn to_bytes(&self) -> Result<u64, String> {
        match self {
            SizeValue::Number(n) => parse_size(&n.to_string()),
            SizeValue::String(s) => parse_size(s),
        }
    }
}

/// Rotation trigger for log files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RotationTrigger {
    /// Never rotate.
    #[default]
    Never,
    /// Rotate based on time period.
    Time {
        /// The time period for rotation.
        period: RotationPeriod,
    },
    /// Rotate based on file size.
    Size {
        /// Maximum file size in bytes before rotation.
        /// Can be specified as a number (defaults to KB) or string with units (K/M/G, case-insensitive).
        /// Examples: 10 (10KB), "5M" (5MB), "1G" (1GB), "2k" (2KB), "3m" (3MB), "4g" (4GB)
        max_size: u64,
        /// Maximum number of files to keep.
        max_files: usize,
    },
    /// Rotate based on both time and size.
    Both {
        /// The time period for rotation.
        period: RotationPeriod,
        /// Maximum file size in bytes before rotation.
        /// Can be specified as a number (defaults to KB) or string with units (K/M/G, case-insensitive).
        /// Examples: 10 (10KB), "5M" (5MB), "1G" (1GB), "2k" (2KB), "3m" (3MB), "4g" (4GB)
        max_size: u64,
        /// Maximum number of files to keep.
        max_files: usize,
    },
}

impl<'de> Deserialize<'de> for RotationTrigger {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum RotationInput {
            Simple(String),
            Complex {
                #[serde(rename = "type")]
                rotation_type: Option<String>,
                period: Option<RotationPeriod>,
                max_size: Option<SizeValue>,
                max_files: Option<usize>,
            },
        }

        let input = RotationInput::deserialize(deserializer)?;

        match input {
            RotationInput::Simple(rotation_type) => match rotation_type.as_str() {
                "never" => Ok(RotationTrigger::Never),
                "size" => Ok(RotationTrigger::Size {
                    max_size: 10 * 1024 * 1024,
                    max_files: 5,
                }),
                "time" => {
                    #[cfg(feature = "time")]
                    {
                        Ok(RotationTrigger::Time {
                            period: RotationPeriod::Daily,
                        })
                    }
                    #[cfg(not(feature = "time"))]
                    {
                        Err(de::Error::custom(
                            "time-based rotation requires time feature",
                        ))
                    }
                }
                "both" => {
                    #[cfg(feature = "time")]
                    {
                        Ok(RotationTrigger::Both {
                            period: RotationPeriod::Daily,
                            max_size: 10 * 1024 * 1024,
                            max_files: 5,
                        })
                    }
                    #[cfg(not(feature = "time"))]
                    {
                        Err(de::Error::custom(
                            "time-based rotation requires time feature",
                        ))
                    }
                }
                other => Err(de::Error::custom(format!(
                    "unknown rotation type: {}",
                    other
                ))),
            },
            RotationInput::Complex {
                rotation_type,
                period,
                max_size,
                max_files,
            } => match rotation_type.as_deref() {
                Some("never") | None => Ok(RotationTrigger::Never),
                Some("time") => {
                    let period = period.ok_or_else(|| {
                        de::Error::custom("period is required for time-based rotation")
                    })?;
                    Ok(RotationTrigger::Time { period })
                }
                Some("size") => {
                    let max_size = max_size
                        .ok_or_else(|| {
                            de::Error::custom("max_size is required for size-based rotation")
                        })?
                        .to_bytes()
                        .map_err(de::Error::custom)?;
                    let max_files = max_files.unwrap_or(5);
                    Ok(RotationTrigger::Size {
                        max_size,
                        max_files,
                    })
                }
                Some("both") => {
                    let period = period.ok_or_else(|| {
                        de::Error::custom("period is required for time+size rotation")
                    })?;
                    let max_size = max_size
                        .ok_or_else(|| {
                            de::Error::custom("max_size is required for time+size rotation")
                        })?
                        .to_bytes()
                        .map_err(de::Error::custom)?;
                    let max_files = max_files.unwrap_or(5);
                    Ok(RotationTrigger::Both {
                        period,
                        max_size,
                        max_files,
                    })
                }
                Some(other) => Err(de::Error::custom(format!(
                    "unknown rotation type: {}",
                    other
                ))),
            },
        }
    }
}

impl RotationTrigger {
    /// Create a size-based rotation trigger.
    pub fn size(max_size: u64, max_files: usize) -> Self {
        Self::Size {
            max_size,
            max_files,
        }
    }

    /// Create a time-based rotation trigger.
    #[cfg(feature = "time")]
    pub fn time(period: RotationPeriod) -> Self {
        Self::Time { period }
    }

    /// Create a hybrid rotation trigger.
    #[cfg(feature = "time")]
    pub fn both(period: RotationPeriod, max_size: u64, max_files: usize) -> Self {
        Self::Both {
            period,
            max_size,
            max_files,
        }
    }

    /// Get the maximum number of files to keep.
    pub fn max_files(&self) -> Option<usize> {
        match self {
            Self::Never => None,
            Self::Time { .. } => None,
            Self::Size { max_files, .. } => Some(*max_files),
            Self::Both { max_files, .. } => Some(*max_files),
        }
    }

    /// Check if this trigger includes size-based rotation.
    pub fn has_size_rotation(&self) -> bool {
        matches!(self, Self::Size { .. } | Self::Both { .. })
    }
}

/// Time periods for log rotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RotationPeriod {
    /// Never rotate.
    Never,
    /// Rotate every hour.
    Hourly,
    /// Rotate every day.
    Daily,
    /// Rotate every week.
    Weekly,
    /// Rotate every month.
    Monthly,
}

impl RotationPeriod {
    /// Get the time suffix for the current period.
    #[cfg(feature = "time")]
    pub fn get_suffix(&self) -> String {
        let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());

        match self {
            Self::Never => String::new(),
            Self::Hourly => now
                .format(&time::format_description::parse("[year]-[month]-[day]T[hour]").unwrap())
                .unwrap(),
            Self::Daily => now
                .format(&time::format_description::parse("[year]-[month]-[day]").unwrap())
                .unwrap(),
            Self::Weekly => {
                let week_start =
                    now - time::Duration::days(now.weekday().number_days_from_monday() as i64);
                week_start
                    .format(&time::format_description::parse("[year]-[month]-[day]").unwrap())
                    .unwrap()
            }
            Self::Monthly => now
                .format(&time::format_description::parse("[year]-[month]").unwrap())
                .unwrap(),
        }
    }

    /// Get the time suffix for the current period (no-op without time feature).
    #[cfg(not(feature = "time"))]
    pub fn get_suffix(&self) -> String {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotation_trigger_max_files() {
        assert_eq!(RotationTrigger::Never.max_files(), None);
        assert_eq!(RotationTrigger::size(1024, 5).max_files(), Some(5));
        #[cfg(feature = "time")]
        assert_eq!(
            RotationTrigger::time(RotationPeriod::Daily).max_files(),
            None
        );
        #[cfg(feature = "time")]
        assert_eq!(
            RotationTrigger::both(RotationPeriod::Daily, 1024, 3).max_files(),
            Some(3)
        );
    }

    #[test]
    fn test_rotation_trigger_deserialize() {
        // Test deserializing "never"
        let yaml = "never";
        let trigger: RotationTrigger = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(trigger, RotationTrigger::Never);

        // Test deserializing size-based rotation with number (defaults to KB)
        let yaml = r#"
type: size
max_size: 10
max_files: 5
"#;
        let trigger: RotationTrigger = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            trigger,
            RotationTrigger::Size {
                max_size: 10 * 1024,
                max_files: 5
            }
        );

        // Test deserializing size-based rotation with KB
        let yaml = r#"
type: size
max_size: "5K"
max_files: 3
"#;
        let trigger: RotationTrigger = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            trigger,
            RotationTrigger::Size {
                max_size: 5 * 1024,
                max_files: 3
            }
        );

        // Test deserializing size-based rotation with MB
        let yaml = r#"
type: size
max_size: "2M"
max_files: 4
"#;
        let trigger: RotationTrigger = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            trigger,
            RotationTrigger::Size {
                max_size: 2 * 1024 * 1024,
                max_files: 4
            }
        );

        // Test deserializing size-based rotation with lowercase units
        let yaml = r#"
type: size
max_size: "3k"
max_files: 6
"#;
        let trigger: RotationTrigger = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            trigger,
            RotationTrigger::Size {
                max_size: 3 * 1024,
                max_files: 6
            }
        );

        // Test deserializing size-based rotation with lowercase MB
        let yaml = r#"
type: size
max_size: "4m"
max_files: 7
"#;
        let trigger: RotationTrigger = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            trigger,
            RotationTrigger::Size {
                max_size: 4 * 1024 * 1024,
                max_files: 7
            }
        );

        // Test deserializing size-based rotation with lowercase GB
        let yaml = r#"
type: size
max_size: "2g"
max_files: 8
"#;
        let trigger: RotationTrigger = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            trigger,
            RotationTrigger::Size {
                max_size: 2 * 1024 * 1024 * 1024,
                max_files: 8
            }
        );

        // Test deserializing time-based rotation
        #[cfg(feature = "time")]
        {
            let yaml = r#"
type: time
period: daily
"#;
            let trigger: RotationTrigger = serde_yaml::from_str(yaml).unwrap();
            assert_eq!(
                trigger,
                RotationTrigger::Time {
                    period: RotationPeriod::Daily
                }
            );
        }

        // Test deserializing both time and size rotation
        #[cfg(feature = "time")]
        {
            let yaml = r#"
type: both
period: hourly
max_size: "512K"
max_files: 10
"#;
            let trigger: RotationTrigger = serde_yaml::from_str(yaml).unwrap();
            assert_eq!(
                trigger,
                RotationTrigger::Both {
                    period: RotationPeriod::Hourly,
                    max_size: 512 * 1024,
                    max_files: 10
                }
            );
        }
    }

    #[cfg(feature = "time")]
    #[test]
    fn test_rotation_period_suffixes() {
        let daily = RotationPeriod::Daily.get_suffix();
        assert!(daily.contains('-'));
        assert_eq!(daily.chars().filter(|c| *c == '-').count(), 2);

        let hourly = RotationPeriod::Hourly.get_suffix();
        assert!(hourly.contains('T'));
        assert!(hourly.contains('-'));

        let weekly = RotationPeriod::Weekly.get_suffix();
        assert!(weekly.contains('-'));

        let monthly = RotationPeriod::Monthly.get_suffix();
        assert!(monthly.contains('-'));
        assert_eq!(monthly.chars().filter(|c| *c == '-').count(), 1);
    }

    #[cfg(feature = "time")]
    #[test]
    fn test_rotation_period_never() {
        assert_eq!(RotationPeriod::Never.get_suffix(), "");
    }

    #[cfg(feature = "time")]
    #[test]
    fn test_rotation_trigger_constructors() {
        let size_trigger = RotationTrigger::size(1024, 5);
        assert_eq!(
            size_trigger,
            RotationTrigger::Size {
                max_size: 1024,
                max_files: 5
            }
        );

        let time_trigger = RotationTrigger::time(RotationPeriod::Hourly);
        assert_eq!(
            time_trigger,
            RotationTrigger::Time {
                period: RotationPeriod::Hourly
            }
        );

        let both_trigger = RotationTrigger::both(RotationPeriod::Daily, 2048, 10);
        assert_eq!(
            both_trigger,
            RotationTrigger::Both {
                period: RotationPeriod::Daily,
                max_size: 2048,
                max_files: 10
            }
        );
    }
}
