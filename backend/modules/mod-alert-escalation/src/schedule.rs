//! On-call schedule management.
//!
//! Manages rotation schedules, shift assignments, and after-hours routing
//! for alert escalation.

use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};

/// A person on rotation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnCallMember {
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}

/// An on-call schedule with rotation rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnCallSchedule {
    pub name: String,
    pub members: Vec<OnCallMember>,
    /// Rotation period in hours (e.g. 24 for daily, 168 for weekly).
    pub rotation_hours: u32,
    /// Business hours start (0–23).
    pub business_hours_start: u32,
    /// Business hours end (0–23).
    pub business_hours_end: u32,
}

impl OnCallSchedule {
    /// Determine if the current time is within business hours.
    pub fn is_business_hours(&self, now: DateTime<Utc>) -> bool {
        let hour = now.hour();
        hour >= self.business_hours_start && hour < self.business_hours_end
    }

    /// Get the current on-call member based on rotation.
    pub fn current_on_call(&self, now: DateTime<Utc>) -> Option<&OnCallMember> {
        if self.members.is_empty() {
            return None;
        }
        let epoch_hours = now.timestamp() as u64 / 3600;
        let index = (epoch_hours / self.rotation_hours as u64) % self.members.len() as u64;
        self.members.get(index as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_business_hours() {
        let schedule = OnCallSchedule {
            name: "Daily rotation".into(),
            members: vec![],
            rotation_hours: 24,
            business_hours_start: 9,
            business_hours_end: 18,
        };

        let morning = Utc::now().date_naive().and_hms_opt(10, 0, 0).unwrap().and_utc();
        assert!(schedule.is_business_hours(morning));

        let night = Utc::now().date_naive().and_hms_opt(22, 0, 0).unwrap().and_utc();
        assert!(!schedule.is_business_hours(night));
    }

    #[test]
    fn test_current_on_call() {
        let schedule = OnCallSchedule {
            name: "Weekly".into(),
            members: vec![
                OnCallMember { name: "Alice".into(), phone: None, email: None },
                OnCallMember { name: "Bob".into(), phone: None, email: None },
            ],
            rotation_hours: 168,
            business_hours_start: 9,
            business_hours_end: 18,
        };

        let on_call = schedule.current_on_call(Utc::now());
        assert!(on_call.is_some());
    }
}
