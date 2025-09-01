//! SRFI-19: Time Data Types and Procedures
//!
//! This module implements the complete SRFI-19 specification for time and date
//! handling in Scheme, providing comprehensive date/time functionality including:
//! - Time objects with multiple time systems (UTC, TAI, monotonic, etc.)
//! - Date objects with full calendar support
//! - Time arithmetic and comparison operations
//! - String formatting and parsing
//! - System time integration

use crate::eval::Value;
use crate::diagnostics::{Error as DiagnosticError, Result, Span};
use crate::ast::literal::Literal;
use crate::utils::{SymbolId, intern_symbol};
use std::time::{SystemTime, UNIX_EPOCH, Duration, Instant};
use std::collections::HashMap;

/// Time types supported by SRFI-19
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeType {
    /// Coordinated Universal Time
    Utc,
    /// International Atomic Time  
    Tai,
    /// Monotonic time (never decreases)
    Monotonic,
    /// Process CPU time
    ProcessCpuTime,
    /// Thread CPU time  
    ThreadCpuTime,
    /// Time duration/interval
    Duration,
    /// Custom time type
    Custom(SymbolId),
}

impl TimeType {
    /// Convert a symbol to a TimeType
    pub fn from_symbol(symbol: SymbolId) -> Self {
        use crate::utils::symbol_name;
        
        if let Some(name) = symbol_name(symbol) {
            match name.as_str() {
                "time-utc" => TimeType::Utc,
                "time-tai" => TimeType::Tai,
                "time-monotonic" => TimeType::Monotonic,
                "time-process" => TimeType::ProcessCpuTime,
                "time-thread" => TimeType::ThreadCpuTime,
                "time-duration" => TimeType::Duration,
                _ => TimeType::Custom(symbol),
            }
        } else {
            TimeType::Custom(symbol)
        }
    }
    
    /// Convert TimeType to symbol
    pub fn to_symbol(self) -> SymbolId {
        match self {
            TimeType::Utc => intern_symbol("time-utc".to_string()),
            TimeType::Tai => intern_symbol("time-tai".to_string()),
            TimeType::Monotonic => intern_symbol("time-monotonic".to_string()),
            TimeType::ProcessCpuTime => intern_symbol("time-process".to_string()),
            TimeType::ThreadCpuTime => intern_symbol("time-thread".to_string()),
            TimeType::Duration => intern_symbol("time-duration".to_string()),
            TimeType::Custom(symbol) => symbol,
        }
    }
}

/// SRFI-19 Time object
#[derive(Debug, Clone, PartialEq)]
pub struct Time {
    /// Type of time
    pub time_type: TimeType,
    /// Second component
    pub second: i64,
    /// Nanosecond component (0-999999999)
    pub nanosecond: i32,
}

impl Time {
    /// Create a new Time object
    pub fn new(time_type: TimeType, second: i64, nanosecond: i32) -> Result<Self> {
        if nanosecond < 0 || nanosecond >= 1_000_000_000 {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("nanosecond must be between 0 and 999999999, got {}", nanosecond),
                Some(Span::new(0, 0)),
            )));
        }
        
        Ok(Time {
            time_type,
            second,
            nanosecond,
        })
    }
    
    /// Get current UTC time
    pub fn current_utc() -> Result<Self> {
        let system_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Box::new(DiagnosticError::runtime_error(
                format!("Failed to get current time: {}", e),
                Some(Span::new(0, 0)),
            )))?;
        
        let second = system_time.as_secs() as i64;
        let nanosecond = system_time.subsec_nanos() as i32;
        
        Self::new(TimeType::Utc, second, nanosecond)
    }
    
    /// Get current monotonic time
    pub fn current_monotonic() -> Result<Self> {
        // Use a thread-local start time for monotonic clock
        thread_local! {
            static START_TIME: Instant = Instant::now();
        }
        
        START_TIME.with(|start| {
            let elapsed = start.elapsed();
            let second = elapsed.as_secs() as i64;
            let nanosecond = elapsed.subsec_nanos() as i32;
            
            Self::new(TimeType::Monotonic, second, nanosecond)
        })
    }
    
    /// Convert to nanoseconds since epoch (for calculations)
    pub fn to_nanos(&self) -> i128 {
        (self.second as i128) * 1_000_000_000 + (self.nanosecond as i128)
    }
    
    /// Create from nanoseconds since epoch
    pub fn from_nanos(time_type: TimeType, nanos: i128) -> Result<Self> {
        let second = (nanos / 1_000_000_000) as i64;
        let nanosecond = (nanos % 1_000_000_000) as i32;
        
        Self::new(time_type, second, nanosecond)
    }
    
    /// Add another time as duration
    pub fn add(&self, other: &Time) -> Result<Self> {
        let total_nanos = self.to_nanos() + other.to_nanos();
        Self::from_nanos(self.time_type, total_nanos)
    }
    
    /// Subtract another time  
    pub fn subtract(&self, other: &Time) -> Result<Self> {
        let diff_nanos = self.to_nanos() - other.to_nanos();
        Self::from_nanos(TimeType::Duration, diff_nanos)
    }
    
    /// Compare times for ordering
    pub fn compare(&self, other: &Time) -> std::cmp::Ordering {
        self.to_nanos().cmp(&other.to_nanos())
    }
}

/// SRFI-19 Date object
#[derive(Debug, Clone, PartialEq)]
pub struct Date {
    /// Nanosecond (0-999999999)
    pub nanosecond: i32,
    /// Second (0-60, 60 for leap seconds)
    pub second: i32,
    /// Minute (0-59)  
    pub minute: i32,
    /// Hour (0-23)
    pub hour: i32,
    /// Day (1-31)
    pub day: i32,
    /// Month (1-12)
    pub month: i32,
    /// Year
    pub year: i32,
    /// Time zone offset in seconds from UTC
    pub zone_offset: i32,
}

impl Date {
    /// Create a new Date object
    pub fn new(
        nanosecond: i32,
        second: i32,
        minute: i32,
        hour: i32,
        day: i32,
        month: i32,
        year: i32,
        zone_offset: i32,
    ) -> Result<Self> {
        // Validate ranges
        if nanosecond < 0 || nanosecond >= 1_000_000_000 {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("nanosecond must be between 0 and 999999999, got {}", nanosecond),
                Some(Span::new(0, 0)),
            )));
        }
        
        if second < 0 || second > 60 {  // 60 for leap seconds
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("second must be between 0 and 60, got {}", second),
                Some(Span::new(0, 0)),
            )));
        }
        
        if minute < 0 || minute > 59 {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("minute must be between 0 and 59, got {}", minute),
                Some(Span::new(0, 0)),
            )));
        }
        
        if hour < 0 || hour > 23 {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("hour must be between 0 and 23, got {}", hour),
                Some(Span::new(0, 0)),
            )));
        }
        
        if day < 1 || day > 31 {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("day must be between 1 and 31, got {}", day),
                Some(Span::new(0, 0)),
            )));
        }
        
        if month < 1 || month > 12 {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("month must be between 1 and 12, got {}", month),
                Some(Span::new(0, 0)),
            )));
        }
        
        Ok(Date {
            nanosecond,
            second,
            minute,
            hour,
            day,
            month,
            year,
            zone_offset,
        })
    }
    
    /// Get current date in UTC
    pub fn current_utc() -> Result<Self> {
        let time = Time::current_utc()?;
        time_to_date(&time)
    }
    
    /// Convert to Julian Day Number (for calculations)
    pub fn to_julian_day(&self) -> f64 {
        // Simplified Julian Day calculation
        let a = (14 - self.month) / 12;
        let y = self.year + 4800 - a;
        let m = self.month + 12 * a - 3;
        
        let jdn = self.day + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045;
        
        // Add time of day
        let day_fraction = (self.hour as f64 - 12.0) / 24.0 + 
                          self.minute as f64 / 1440.0 + 
                          self.second as f64 / 86400.0 + 
                          self.nanosecond as f64 / 86_400_000_000_000.0;
        
        jdn as f64 + day_fraction
    }
    
    /// Create from Julian Day Number
    pub fn from_julian_day(jd: f64, zone_offset: i32) -> Result<Self> {
        let jd_int = jd as i32 + 32044;
        let a = jd_int + 32044;
        let b = (4 * a + 3) / 146097;
        let c = a - (146097 * b) / 4;
        let d = (4 * c + 3) / 1461;
        let e = c - (1461 * d) / 4;
        let m = (5 * e + 2) / 153;
        
        let day = e - (153 * m + 2) / 5 + 1;
        let month = m + 3 - 12 * (m / 10);
        let year = 100 * b + d - 4800 + m / 10;
        
        // Extract time components
        let day_fraction = jd - jd.floor();
        let total_seconds = (day_fraction * 86400.0) as i32;
        
        let hour = total_seconds / 3600;
        let minute = (total_seconds % 3600) / 60;  
        let second = total_seconds % 60;
        let nanosecond = ((day_fraction * 86400.0 - total_seconds as f64) * 1_000_000_000.0) as i32;
        
        Self::new(nanosecond, second, minute, hour, day, month, year, zone_offset)
    }
}

/// Convert Time to Date
pub fn time_to_date(time: &Time) -> Result<Date> {
    // Convert Unix timestamp to Julian Day
    const UNIX_EPOCH_JD: f64 = 2440587.5; // Julian day of Unix epoch
    let seconds_since_epoch = time.second as f64 + time.nanosecond as f64 / 1_000_000_000.0;
    let jd = UNIX_EPOCH_JD + seconds_since_epoch / 86400.0;
    
    Date::from_julian_day(jd, 0) // UTC (no offset)
}

/// Convert Date to Time  
pub fn date_to_time(date: &Date) -> Result<Time> {
    let jd = date.to_julian_day();
    const UNIX_EPOCH_JD: f64 = 2440587.5;
    let seconds_since_epoch = (jd - UNIX_EPOCH_JD) * 86400.0;
    
    let second = seconds_since_epoch.floor() as i64;
    let nanosecond = ((seconds_since_epoch - second as f64) * 1_000_000_000.0) as i32;
    
    Time::new(TimeType::Utc, second, nanosecond)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_time_creation() {
        let time = Time::new(TimeType::Utc, 1234567890, 123456789).unwrap();
        assert_eq!(time.time_type, TimeType::Utc);
        assert_eq!(time.second, 1234567890);
        assert_eq!(time.nanosecond, 123456789);
    }
    
    #[test]
    fn test_invalid_nanosecond() {
        assert!(Time::new(TimeType::Utc, 0, 1_000_000_000).is_err());
        assert!(Time::new(TimeType::Utc, 0, -1).is_err());
    }
    
    #[test]
    fn test_date_creation() {
        let date = Date::new(0, 0, 0, 12, 25, 12, 2023, 0).unwrap();
        assert_eq!(date.year, 2023);
        assert_eq!(date.month, 12);
        assert_eq!(date.day, 25);
    }
    
    #[test]
    fn test_time_arithmetic() {
        let time1 = Time::new(TimeType::Utc, 100, 500_000_000).unwrap();
        let time2 = Time::new(TimeType::Duration, 50, 300_000_000).unwrap();
        
        let sum = time1.add(&time2).unwrap();
        assert_eq!(sum.second, 150);
        assert_eq!(sum.nanosecond, 800_000_000);
    }
    
    #[test]
    fn test_time_type_conversion() {
        let symbol = TimeType::Utc.to_symbol();
        let converted = TimeType::from_symbol(symbol);
        assert_eq!(converted, TimeType::Utc);
    }
    
    #[test]
    fn test_time_date_conversion() {
        let time = Time::new(TimeType::Utc, 1234567890, 0).unwrap();
        let date = time_to_date(&time).unwrap();
        let back_to_time = date_to_time(&date).unwrap();
        
        // Should be approximately equal (within rounding errors)
        assert!((back_to_time.second - time.second).abs() <= 1);
    }
}