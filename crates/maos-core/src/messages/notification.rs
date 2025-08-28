//! Notification message types for TTS and user alerts
//!
//! This module provides notification types for user alerts, task completions,
//! and system events. Notifications can be formatted for text-to-speech (TTS)
//! with personalized engineer names.

use crate::SessionId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Notification message for TTS and logging
///
/// Represents a notification that can be displayed to the user and/or
/// announced via text-to-speech. Includes urgency levels for prioritization.
///
/// # Example
///
/// ```
/// use maos_core::messages::{NotificationMessage, NotificationType, NotificationUrgency};
/// use maos_core::SessionId;
/// use chrono::Utc;
///
/// let notification = NotificationMessage {
///     message: "Build completed successfully".to_string(),
///     notification_type: NotificationType::TaskCompletion,
///     engineer_name: Some("Marvin".to_string()),
///     session_id: None,
///     urgency: NotificationUrgency::Normal,
///     timestamp: Utc::now(),
/// };
///
/// // Format for TTS
/// let tts = notification.to_tts_string();
/// assert!(tts.contains("Marvin"));
/// assert!(tts.contains("task completed"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    /// The notification message content
    pub message: String,

    /// Type of notification
    pub notification_type: NotificationType,

    /// Optional engineer name for personalization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engineer_name: Option<String>,

    /// Optional session ID for context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<SessionId>,

    /// Urgency level for prioritization
    pub urgency: NotificationUrgency,

    /// When this notification was created
    #[serde(default = "chrono::Utc::now")]
    pub timestamp: DateTime<Utc>,
}

impl NotificationMessage {
    /// Format the notification for text-to-speech announcement
    ///
    /// Formats the message with appropriate prefix based on notification type
    /// and includes the engineer's name if available.
    ///
    /// # Example
    ///
    /// ```
    /// # use maos_core::messages::{NotificationMessage, NotificationType, NotificationUrgency};
    /// # use chrono::Utc;
    /// let mut notification = NotificationMessage {
    ///     message: "tests passed".to_string(),
    ///     notification_type: NotificationType::TaskCompletion,
    ///     engineer_name: Some("Marvin".to_string()),
    ///     session_id: None,
    ///     urgency: NotificationUrgency::Normal,
    ///     timestamp: Utc::now(),
    /// };
    ///
    /// assert_eq!(notification.to_tts_string(), "Marvin, task completed: tests passed");
    ///
    /// notification.engineer_name = None;
    /// assert_eq!(notification.to_tts_string(), "Engineer, task completed: tests passed");
    /// ```
    pub fn to_tts_string(&self) -> String {
        let engineer = self.engineer_name.as_deref().unwrap_or("Engineer");

        match self.notification_type {
            NotificationType::UserInputRequest => {
                format!("{}, I need your input: {}", engineer, self.message)
            }
            NotificationType::TaskCompletion => {
                format!("{}, task completed: {}", engineer, self.message)
            }
            NotificationType::AgentSpawned => {
                format!("New agent spawned: {}", self.message)
            }
            NotificationType::AgentCompleted => {
                format!("Agent finished: {}", self.message)
            }
            NotificationType::SecurityAlert => {
                format!("Security alert! {}", self.message)
            }
            NotificationType::SystemError => {
                format!("System error: {}", self.message)
            }
        }
    }
}

/// Types of notifications
///
/// Different notification types trigger different TTS formatting and
/// may have different handling in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    /// User input is required
    UserInputRequest,

    /// A task has been completed
    TaskCompletion,

    /// A new agent has been spawned
    AgentSpawned,

    /// An agent has completed its work
    AgentCompleted,

    /// Security issue detected
    SecurityAlert,

    /// System error occurred
    SystemError,
}

/// Urgency levels for notifications
///
/// Used to prioritize notifications and determine if they should
/// interrupt the user's workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationUrgency {
    /// Low priority - can be batched/delayed
    Low,

    /// Normal priority - standard notification
    Normal,

    /// High priority - needs immediate attention
    High,

    /// Critical priority - interrupt current work
    Critical,
}

impl Default for NotificationUrgency {
    fn default() -> Self {
        Self::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SessionId;
    use serde_json::json;
    use std::str::FromStr;

    #[test]
    fn test_notification_message_serialization() {
        let notification = NotificationMessage {
            message: "Build completed successfully".to_string(),
            notification_type: NotificationType::TaskCompletion,
            engineer_name: Some("Marvin".to_string()),
            session_id: Some(
                SessionId::from_str("sess_12345678-1234-1234-1234-123456789012").unwrap(),
            ),
            urgency: NotificationUrgency::Normal,
            timestamp: chrono::Utc::now(),
        };

        // Should serialize to JSON
        let json = serde_json::to_value(&notification).unwrap();
        assert_eq!(json["message"], "Build completed successfully");
        assert_eq!(json["notification_type"], "task_completion");
        assert_eq!(json["urgency"], "normal");
        assert_eq!(json["engineer_name"], "Marvin");

        // Should deserialize from JSON
        let deserialized: NotificationMessage = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.message, notification.message);
    }

    #[test]
    fn test_notification_types() {
        assert_eq!(
            serde_json::to_value(NotificationType::UserInputRequest).unwrap(),
            json!("user_input_request")
        );
        assert_eq!(
            serde_json::to_value(NotificationType::TaskCompletion).unwrap(),
            json!("task_completion")
        );
        assert_eq!(
            serde_json::to_value(NotificationType::AgentSpawned).unwrap(),
            json!("agent_spawned")
        );
        assert_eq!(
            serde_json::to_value(NotificationType::AgentCompleted).unwrap(),
            json!("agent_completed")
        );
        assert_eq!(
            serde_json::to_value(NotificationType::SecurityAlert).unwrap(),
            json!("security_alert")
        );
        assert_eq!(
            serde_json::to_value(NotificationType::SystemError).unwrap(),
            json!("system_error")
        );
    }

    #[test]
    fn test_notification_urgency_levels() {
        assert_eq!(
            serde_json::to_value(NotificationUrgency::Low).unwrap(),
            json!("low")
        );
        assert_eq!(
            serde_json::to_value(NotificationUrgency::Normal).unwrap(),
            json!("normal")
        );
        assert_eq!(
            serde_json::to_value(NotificationUrgency::High).unwrap(),
            json!("high")
        );
        assert_eq!(
            serde_json::to_value(NotificationUrgency::Critical).unwrap(),
            json!("critical")
        );
    }

    #[test]
    fn test_tts_formatting_with_engineer() {
        let notification = NotificationMessage {
            message: "tests passed".to_string(),
            notification_type: NotificationType::TaskCompletion,
            engineer_name: Some("Marvin".to_string()),
            session_id: None,
            urgency: NotificationUrgency::Normal,
            timestamp: chrono::Utc::now(),
        };

        let tts = notification.to_tts_string();
        assert_eq!(tts, "Marvin, task completed: tests passed");
    }

    #[test]
    fn test_tts_formatting_without_engineer() {
        let notification = NotificationMessage {
            message: "need your input on PR review".to_string(),
            notification_type: NotificationType::UserInputRequest,
            engineer_name: None,
            session_id: None,
            urgency: NotificationUrgency::High,
            timestamp: chrono::Utc::now(),
        };

        let tts = notification.to_tts_string();
        assert_eq!(
            tts,
            "Engineer, I need your input: need your input on PR review"
        );
    }

    #[test]
    fn test_tts_formatting_all_types() {
        // Test each notification type
        let types_and_expected = vec![
            (
                NotificationType::UserInputRequest,
                "test",
                "Marv, I need your input: test",
            ),
            (
                NotificationType::TaskCompletion,
                "done",
                "Marv, task completed: done",
            ),
            (
                NotificationType::AgentSpawned,
                "backend",
                "New agent spawned: backend",
            ),
            (
                NotificationType::AgentCompleted,
                "frontend",
                "Agent finished: frontend",
            ),
            (
                NotificationType::SecurityAlert,
                "danger",
                "Security alert! danger",
            ),
            (
                NotificationType::SystemError,
                "crash",
                "System error: crash",
            ),
        ];

        for (notification_type, message, expected) in types_and_expected {
            let notification = NotificationMessage {
                message: message.to_string(),
                notification_type,
                engineer_name: Some("Marv".to_string()),
                session_id: None,
                urgency: NotificationUrgency::Normal,
                timestamp: chrono::Utc::now(),
            };

            assert_eq!(notification.to_tts_string(), expected);
        }
    }
}
