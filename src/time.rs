use chrono::TimeZone;
use lazy_static::lazy_static;
use regex::Regex;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::Add;

#[derive(Debug)]
pub struct HourMinute {
    hour: u32,
    minute: u32,
}

impl HourMinute {
    pub fn new(hour: u32, minute: u32) -> Option<Self> {
        if (0..=23).contains(&hour) && (0..=59).contains(&minute) {
            Some(Self { hour, minute })
        } else {
            None
        }
    }
}

impl Serialize for HourMinute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{:02}:{:02}", self.hour, self.minute))
    }
}

impl<'de> Deserialize<'de> for HourMinute {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = String::deserialize(deserializer)?;
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d?\d):(\d\d)$").unwrap();
        }
        if let Some(captured) = RE.captures(&val) {
            if let (Some(h), Some(m)) = (captured.get(1), captured.get(2)) {
                if let Some(hm) =
                    HourMinute::new(h.as_str().parse().unwrap(), m.as_str().parse().unwrap())
                {
                    return Ok(hm);
                } else {
                    return Err(D::Error::custom(format!("Time is out of range: '{}'", val)));
                }
            }
        }
        Err(D::Error::custom(format!(
            "Failed to parse time from string: '{}'",
            val
        )))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Duration(i64);

impl Duration {
    pub fn from_minutes(minutes: i64) -> Self {
        Self(minutes * 60)
    }
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(i64);

impl Timestamp {
    pub const ZERO: Self = Self(0);
    pub fn from_date_hm(date: &LocalDate, hm: &HourMinute) -> Self {
        Self(date.and_hms(hm.hour, hm.minute, 0).timestamp())
    }
    pub fn from_date_hm_opt(date: &LocalDate, hm: &Option<HourMinute>) -> Option<Self> {
        if let Some(hm) = hm {
            Some(Self(date.and_hms(hm.hour, hm.minute, 0).timestamp()))
        } else {
            None
        }
    }
    pub fn now() -> Self {
        Self(chrono::Local::now().timestamp())
    }
    pub fn get_date(self) -> LocalDate {
        chrono::Local.timestamp(self.0, 0).date()
    }
}

impl Add<Duration> for Timestamp {
    type Output = Self;
    fn add(self, rhs: Duration) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

pub type LocalDate = chrono::Date<chrono::Local>;
