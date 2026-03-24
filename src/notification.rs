//! Toast notifications and persistent notification log.
//!
//! Provides [`Toast`] for ephemeral overlay notifications with auto-expiry,
//! and [`NotificationLog`] for persistent console/log panel display.
//! Extracted from shruti's toast system.
//!
//! # Toasts vs Log
//!
//! [`Toasts`] manages short-lived overlay messages (save complete, error alerts)
//! that expire after a duration based on severity. [`NotificationLog`] keeps a
//! capped history of all notifications for a scrollable console panel.

use std::borrow::Cow;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Severity level for notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum Severity {
    /// Informational — operation succeeded, status update.
    Info,
    /// Something unexpected but non-fatal.
    Warning,
    /// Operation failed or critical issue.
    Error,
}

impl Severity {
    /// Default display duration for this severity level.
    #[must_use]
    #[inline]
    pub fn default_duration(self) -> Duration {
        match self {
            Severity::Info => Duration::from_secs(3),
            Severity::Warning => Duration::from_secs(5),
            Severity::Error => Duration::from_secs(8),
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

// ---------------------------------------------------------------------------
// Toast — ephemeral overlay notification
// ---------------------------------------------------------------------------

/// An ephemeral notification with auto-expiry.
///
/// Tracks its own creation time and duration so consumers can query
/// [`is_expired`](Toast::is_expired) and [`progress`](Toast::progress)
/// for rendering fade-out animations.
#[derive(Debug, Clone)]
pub struct Toast {
    /// The notification message.
    pub message: String,
    /// Severity level (determines default duration and display style).
    pub severity: Severity,
    created: Instant,
    duration: Duration,
}

impl Toast {
    /// Create a toast with severity-based default duration.
    #[must_use]
    pub fn new(message: impl Into<String>, severity: Severity) -> Self {
        let duration = severity.default_duration();
        Self {
            message: message.into(),
            severity,
            created: Instant::now(),
            duration,
        }
    }

    /// Create a toast with a custom duration.
    #[must_use]
    pub fn with_duration(
        message: impl Into<String>,
        severity: Severity,
        duration: Duration,
    ) -> Self {
        Self {
            message: message.into(),
            severity,
            created: Instant::now(),
            duration,
        }
    }

    /// Whether this toast has expired and should be removed.
    #[must_use]
    #[inline]
    pub fn is_expired(&self) -> bool {
        self.created.elapsed() >= self.duration
    }

    /// Progress from 0.0 (just created) to 1.0 (expired).
    ///
    /// Useful for fade-out animations. Returns 1.0 for zero-duration toasts.
    #[must_use]
    #[inline]
    pub fn progress(&self) -> f32 {
        let total = self.duration.as_secs_f32();
        if total == 0.0 {
            return 1.0;
        }
        let elapsed = self.created.elapsed().as_secs_f32();
        (elapsed / total).clamp(0.0, 1.0)
    }

    /// Remaining time before expiry.
    #[must_use]
    #[inline]
    pub fn remaining(&self) -> Duration {
        self.duration.saturating_sub(self.created.elapsed())
    }

    /// The total duration this toast will be displayed.
    #[must_use]
    #[inline]
    pub fn duration(&self) -> Duration {
        self.duration
    }
}

// ---------------------------------------------------------------------------
// Toasts — active toast manager
// ---------------------------------------------------------------------------

/// Manages a collection of active toast notifications.
///
/// Call [`gc`](Toasts::gc) each frame to remove expired toasts.
#[derive(Debug, Clone, Default)]
pub struct Toasts {
    active: Vec<Toast>,
}

impl Toasts {
    /// Create an empty toast manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a toast with severity-based default duration.
    pub fn push(&mut self, message: impl Into<String>, severity: Severity) {
        let toast = Toast::new(message, severity);
        tracing::debug!(severity = %toast.severity, msg = %toast.message, "toast pushed");
        self.active.push(toast);
    }

    /// Push a pre-built toast.
    pub fn push_toast(&mut self, toast: Toast) {
        tracing::debug!(severity = %toast.severity, msg = %toast.message, "toast pushed");
        self.active.push(toast);
    }

    /// Remove expired toasts. Call each frame.
    pub fn gc(&mut self) {
        let before = self.active.len();
        self.active.retain(|t| !t.is_expired());
        let removed = before - self.active.len();
        if removed > 0 {
            tracing::trace!(removed, remaining = self.active.len(), "toasts gc");
        }
    }

    /// Active (non-expired) toasts, newest last.
    #[must_use]
    #[inline]
    pub fn active(&self) -> &[Toast] {
        &self.active
    }

    /// Number of active toasts.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.active.len()
    }

    /// Whether there are no active toasts.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.active.is_empty()
    }

    /// Remove all active toasts.
    pub fn clear(&mut self) {
        self.active.clear();
        tracing::debug!("toasts cleared");
    }
}

// ---------------------------------------------------------------------------
// Notification — persistent log entry
// ---------------------------------------------------------------------------

/// A persistent notification for console/log panel display.
///
/// Unlike [`Toast`], notifications don't expire. They use a monotonic
/// sequence number instead of wall-clock time so ordering is always correct.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Notification {
    /// The notification message.
    pub message: String,
    /// Severity level.
    pub severity: Severity,
    /// Source subsystem (e.g. "history", "export", "inspector").
    pub source: Cow<'static, str>,
    /// Monotonic sequence number (assigned by [`NotificationLog`]).
    pub seq: u64,
}

// ---------------------------------------------------------------------------
// NotificationLog — capped history for console panel
// ---------------------------------------------------------------------------

/// Default maximum log entries.
const DEFAULT_MAX_LOG: usize = 512;

/// Capped notification history for a scrollable console/log panel.
///
/// Uses [`VecDeque`] for O(1) eviction of oldest entries.
#[derive(Debug, Clone)]
pub struct NotificationLog {
    entries: VecDeque<Notification>,
    max_entries: usize,
    next_seq: u64,
}

impl NotificationLog {
    /// Create a log with default max entries (512).
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries: DEFAULT_MAX_LOG,
            next_seq: 0,
        }
    }

    /// Create a log with a custom max entries cap.
    #[must_use]
    pub fn with_max_entries(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_entries.min(1024)),
            max_entries,
            next_seq: 0,
        }
    }

    /// Add a notification to the log.
    pub fn push(&mut self, message: impl Into<String>, severity: Severity, source: &'static str) {
        self.push_notification(message.into(), severity, Cow::Borrowed(source));
    }

    /// Add a notification with a dynamic source string.
    pub fn push_owned(&mut self, message: impl Into<String>, severity: Severity, source: String) {
        self.push_notification(message.into(), severity, Cow::Owned(source));
    }

    fn push_notification(
        &mut self,
        message: String,
        severity: Severity,
        source: Cow<'static, str>,
    ) {
        let notification = Notification {
            message,
            severity,
            source,
            seq: self.next_seq,
        };
        self.next_seq += 1;
        tracing::debug!(
            seq = notification.seq,
            severity = %notification.severity,
            source = %notification.source,
            "notification logged"
        );
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(notification);
    }

    /// All log entries, oldest first.
    #[must_use]
    #[inline]
    pub fn entries(&self) -> &VecDeque<Notification> {
        &self.entries
    }

    /// Number of entries in the log.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the log is empty.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Filter entries by severity.
    #[must_use]
    pub fn by_severity(&self, severity: Severity) -> Vec<&Notification> {
        self.entries
            .iter()
            .filter(|n| n.severity == severity)
            .collect()
    }

    /// Filter entries by source.
    #[must_use]
    pub fn by_source(&self, source: &str) -> Vec<&Notification> {
        self.entries.iter().filter(|n| n.source == source).collect()
    }

    /// Maximum entries cap.
    #[must_use]
    #[inline]
    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    /// Clear the log.
    pub fn clear(&mut self) {
        self.entries.clear();
        tracing::debug!("notification log cleared");
    }
}

impl Default for NotificationLog {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // === Severity ===

    #[test]
    fn severity_default_durations() {
        assert_eq!(Severity::Info.default_duration(), Duration::from_secs(3));
        assert_eq!(Severity::Warning.default_duration(), Duration::from_secs(5));
        assert_eq!(Severity::Error.default_duration(), Duration::from_secs(8));
    }

    #[test]
    fn severity_display() {
        assert_eq!(Severity::Info.to_string(), "info");
        assert_eq!(Severity::Warning.to_string(), "warning");
        assert_eq!(Severity::Error.to_string(), "error");
    }

    // === Toast ===

    #[test]
    fn toast_new() {
        let t = Toast::new("hello", Severity::Info);
        assert_eq!(t.message, "hello");
        assert_eq!(t.severity, Severity::Info);
        assert!(!t.is_expired());
        assert!(t.progress() < 0.1);
    }

    #[test]
    fn toast_with_custom_duration() {
        let t = Toast::with_duration("fast", Severity::Warning, Duration::from_millis(1));
        assert_eq!(t.duration(), Duration::from_millis(1));
        // Will expire almost immediately
        std::thread::sleep(Duration::from_millis(2));
        assert!(t.is_expired());
        assert!((t.progress() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn toast_zero_duration_progress() {
        let t = Toast::with_duration("instant", Severity::Info, Duration::ZERO);
        assert!((t.progress() - 1.0).abs() < f32::EPSILON);
        assert!(t.is_expired());
    }

    #[test]
    fn toast_remaining() {
        let t = Toast::new("test", Severity::Error);
        let remaining = t.remaining();
        assert!(remaining <= Duration::from_secs(8));
        assert!(remaining > Duration::from_secs(7));
    }

    // === Toasts ===

    #[test]
    fn toasts_push_and_len() {
        let mut toasts = Toasts::new();
        assert!(toasts.is_empty());

        toasts.push("a", Severity::Info);
        toasts.push("b", Severity::Warning);
        assert_eq!(toasts.len(), 2);
        assert_eq!(toasts.active()[0].message, "a");
    }

    #[test]
    fn toasts_gc_removes_expired() {
        let mut toasts = Toasts::new();
        toasts.push_toast(Toast::with_duration(
            "fast",
            Severity::Info,
            Duration::from_millis(1),
        ));
        toasts.push("slow", Severity::Error);
        assert_eq!(toasts.len(), 2);

        std::thread::sleep(Duration::from_millis(2));
        toasts.gc();

        assert_eq!(toasts.len(), 1);
        assert_eq!(toasts.active()[0].message, "slow");
    }

    #[test]
    fn toasts_clear() {
        let mut toasts = Toasts::new();
        toasts.push("a", Severity::Info);
        toasts.push("b", Severity::Info);
        toasts.clear();
        assert!(toasts.is_empty());
    }

    // === Notification ===

    #[test]
    fn notification_serde_roundtrip() {
        let n = Notification {
            message: "test".into(),
            severity: Severity::Warning,
            source: Cow::Borrowed("history"),
            seq: 42,
        };
        let json = serde_json::to_string(&n).unwrap();
        let n2: Notification = serde_json::from_str(&json).unwrap();
        assert_eq!(n2.message, "test");
        assert_eq!(n2.severity, Severity::Warning);
        assert_eq!(n2.seq, 42);
    }

    // === NotificationLog ===

    #[test]
    fn log_push_and_query() {
        let mut log = NotificationLog::new();
        log.push("save ok", Severity::Info, "history");
        log.push("low mem", Severity::Warning, "hw");
        log.push("crash", Severity::Error, "export");

        assert_eq!(log.len(), 3);
        assert_eq!(log.entries()[0].seq, 0);
        assert_eq!(log.entries()[2].seq, 2);
    }

    #[test]
    fn log_by_severity() {
        let mut log = NotificationLog::new();
        log.push("a", Severity::Info, "src");
        log.push("b", Severity::Error, "src");
        log.push("c", Severity::Info, "src");

        let infos = log.by_severity(Severity::Info);
        assert_eq!(infos.len(), 2);
        let errors = log.by_severity(Severity::Error);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn log_by_source() {
        let mut log = NotificationLog::new();
        log.push("a", Severity::Info, "history");
        log.push("b", Severity::Info, "export");
        log.push("c", Severity::Info, "history");

        let history = log.by_source("history");
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn log_max_entries_eviction() {
        let mut log = NotificationLog::with_max_entries(3);
        for i in 0..5 {
            log.push(format!("msg {i}"), Severity::Info, "test");
        }
        assert_eq!(log.len(), 3);
        assert_eq!(log.max_entries(), 3);
        // Oldest evicted, newest kept
        assert_eq!(log.entries()[0].message, "msg 2");
        assert_eq!(log.entries()[2].message, "msg 4");
    }

    #[test]
    fn log_clear() {
        let mut log = NotificationLog::new();
        log.push("a", Severity::Info, "test");
        log.clear();
        assert!(log.is_empty());
    }

    #[test]
    fn log_default() {
        let log = NotificationLog::default();
        assert_eq!(log.max_entries(), DEFAULT_MAX_LOG);
        assert!(log.is_empty());
    }

    #[test]
    fn log_push_owned_source() {
        let mut log = NotificationLog::new();
        log.push_owned("msg", Severity::Info, "dynamic_source".to_string());
        assert_eq!(log.entries()[0].source, "dynamic_source");
    }

    #[test]
    fn log_sequence_numbers_monotonic() {
        let mut log = NotificationLog::new();
        log.push("a", Severity::Info, "test");
        log.push("b", Severity::Info, "test");
        log.push("c", Severity::Info, "test");
        assert!(log.entries()[0].seq < log.entries()[1].seq);
        assert!(log.entries()[1].seq < log.entries()[2].seq);
    }
}
