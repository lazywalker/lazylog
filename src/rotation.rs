use serde::{Deserialize, Serialize};
#[cfg(feature = "time")]
use time::OffsetDateTime;

/// Rotation trigger for log files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RotationTrigger {
    /// Never rotate.
    #[default]
    Never,
    /// Rotate based on time period.
    Time { period: RotationPeriod },
    /// Rotate based on file size.
    Size { max_size: u64, max_files: usize },
    /// Rotate based on both time and size.
    Both {
        period: RotationPeriod,
        max_size: u64,
        max_files: usize,
    },
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
    fn test_rotation_trigger_has_size_rotation() {
        assert!(!RotationTrigger::Never.has_size_rotation());
        assert!(RotationTrigger::size(1024, 5).has_size_rotation());
        #[cfg(feature = "time")]
        assert!(!RotationTrigger::time(RotationPeriod::Daily).has_size_rotation());
        #[cfg(feature = "time")]
        assert!(RotationTrigger::both(RotationPeriod::Daily, 1024, 3).has_size_rotation());
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
