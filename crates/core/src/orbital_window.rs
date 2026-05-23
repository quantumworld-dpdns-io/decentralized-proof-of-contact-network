use chrono::{DateTime, Duration, Timelike, Utc};
use chrono::Timelike;
use uuid::Uuid;

use crate::types::{OrbitalWindow, WindowType};

impl OrbitalWindow {
    pub fn new(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        window_type: WindowType,
    ) -> Self {
        OrbitalWindow {
            id: Uuid::new_v4(),
            start_time: start,
            end_time: end,
            window_type,
        }
    }

    pub fn duration(&self) -> Duration {
        self.end_time.signed_duration_since(self.start_time)
    }

    pub fn is_active(&self) -> bool {
        let now = Utc::now();
        self.start_time <= now && now <= self.end_time
    }

    pub fn is_active_at(&self, time: DateTime<Utc>) -> bool {
        self.start_time <= time && time <= self.end_time
    }

    pub fn overlaps_with(&self, other: &OrbitalWindow) -> bool {
        self.start_time <= other.end_time && other.start_time <= self.end_time
    }

    pub fn contains(&self, time: DateTime<Utc>) -> bool {
        self.start_time <= time && time <= self.end_time
    }

    pub fn start_time(&self) -> DateTime<Utc> {
        self.start_time
    }

    pub fn end_time(&self) -> DateTime<Utc> {
        self.end_time
    }
}

pub struct OrbitalWindowBuilder {
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    window_type: WindowType,
    id: Option<Uuid>,
}

impl OrbitalWindowBuilder {
    pub fn new() -> Self {
        OrbitalWindowBuilder {
            start_time: None,
            end_time: None,
            window_type: WindowType::Standard,
            id: None,
        }
    }

    pub fn start_time(mut self, start: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self
    }

    pub fn end_time(mut self, end: DateTime<Utc>) -> Self {
        self.end_time = Some(end);
        self
    }

    pub fn window_type(mut self, wtype: WindowType) -> Self {
        self.window_type = wtype;
        self
    }

    pub fn id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    pub fn duration_from_now(mut self, duration: Duration) -> Self {
        let now = Utc::now();
        self.start_time = Some(now);
        self.end_time = Some(now + duration);
        self
    }

    pub fn build(self) -> OrbitalWindow {
        let now = Utc::now();
        OrbitalWindow {
            id: self.id.unwrap_or_else(Uuid::new_v4),
            start_time: self.start_time.unwrap_or(now),
            end_time: self.end_time.unwrap_or(now + Duration::hours(1)),
            window_type: self.window_type,
        }
    }
}

impl Default for OrbitalWindowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn hourly_window() -> OrbitalWindow {
    let now = Utc::now();
    let start = now
        .date_naive()
        .and_hms_opt(now.hour(), 0, 0)
        .unwrap()
        .and_utc();
    OrbitalWindow::new(start, start + Duration::hours(1), WindowType::Standard)
}

pub fn daily_window() -> OrbitalWindow {
    let now = Utc::now();
    let start = now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    OrbitalWindow::new(start, start + Duration::days(1), WindowType::Standard)
}

pub fn extended_window(hours: i64) -> OrbitalWindow {
    let now = Utc::now();
    OrbitalWindow::new(now, now + Duration::hours(hours), WindowType::Extended)
}

pub fn emergency_window(duration_minutes: i64) -> OrbitalWindow {
    let now = Utc::now();
    OrbitalWindow::new(
        now,
        now + Duration::minutes(duration_minutes),
        WindowType::Emergency,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_orbital_window_new() {
        let start = Utc::now();
        let end = start + Duration::hours(2);
        let window = OrbitalWindow::new(start, end, WindowType::Standard);
        assert_eq!(window.start_time, start);
        assert_eq!(window.end_time, end);
        assert_eq!(window.window_type, WindowType::Standard);
    }

    #[test]
    fn test_duration() {
        let start = Utc::now();
        let end = start + Duration::hours(3);
        let window = OrbitalWindow::new(start, end, WindowType::Standard);
        assert_eq!(window.duration(), Duration::hours(3));
    }

    #[test]
    fn test_is_active() {
        let past = Utc::now() - Duration::hours(2);
        let future = Utc::now() + Duration::hours(2);
        let window = OrbitalWindow::new(past, future, WindowType::Standard);
        assert!(window.is_active());
    }

    #[test]
    fn test_is_not_active() {
        let past = Utc::now() - Duration::hours(4);
        let earlier = Utc::now() - Duration::hours(2);
        let window = OrbitalWindow::new(past, earlier, WindowType::Standard);
        assert!(!window.is_active());
    }

    #[test]
    fn test_contains() {
        let start = Utc::now() - Duration::hours(1);
        let end = Utc::now() + Duration::hours(1);
        let window = OrbitalWindow::new(start, end, WindowType::Standard);
        assert!(window.contains(Utc::now()));
        assert!(window.contains(start));
        assert!(window.contains(end));
        assert!(!window.contains(end + Duration::seconds(1)));
    }

    #[test]
    fn test_overlaps() {
        let w1 = OrbitalWindow::new(
            Utc::now(),
            Utc::now() + Duration::hours(2),
            WindowType::Standard,
        );
        let w2 = OrbitalWindow::new(
            Utc::now() + Duration::hours(1),
            Utc::now() + Duration::hours(3),
            WindowType::Standard,
        );
        assert!(w1.overlaps_with(&w2));
    }

    #[test]
    fn test_no_overlap() {
        let w1 = OrbitalWindow::new(
            Utc::now(),
            Utc::now() + Duration::hours(1),
            WindowType::Standard,
        );
        let w2 = OrbitalWindow::new(
            Utc::now() + Duration::hours(2),
            Utc::now() + Duration::hours(3),
            WindowType::Standard,
        );
        assert!(!w1.overlaps_with(&w2));
    }

    #[test]
    fn test_builder() {
        let window = OrbitalWindowBuilder::new()
            .start_time(Utc::now())
            .end_time(Utc::now() + Duration::hours(4))
            .window_type(WindowType::Extended)
            .build();
        assert_eq!(window.window_type, WindowType::Extended);
        assert_eq!(window.duration(), Duration::hours(4));
    }

    #[test]
    fn test_hourly_window() {
        let w = hourly_window();
        assert_eq!(w.window_type, WindowType::Standard);
        assert_eq!(w.duration(), Duration::hours(1));
    }

    #[test]
    fn test_daily_window() {
        let w = daily_window();
        assert_eq!(w.window_type, WindowType::Standard);
        assert_eq!(w.duration(), Duration::days(1));
    }

    #[test]
    fn test_extended_window() {
        let w = extended_window(6);
        assert_eq!(w.window_type, WindowType::Extended);
        assert_eq!(w.duration(), Duration::hours(6));
    }

    #[test]
    fn test_emergency_window() {
        let w = emergency_window(30);
        assert_eq!(w.window_type, WindowType::Emergency);
        assert_eq!(w.duration(), Duration::minutes(30));
    }
}
