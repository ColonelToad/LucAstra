//! Conversation management for multi-turn LLM interactions.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ConversationError {
    #[error("conversation not found: {0}")]
    NotFound(String),
    #[error("invalid message: {0}")]
    InvalidMessage(String),
}

pub type ConversationResult<T> = std::result::Result<T, ConversationError>;

/// Message role in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// A single message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    #[serde(default = "default_timestamp")]
    pub timestamp: i64,
}

fn default_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

impl Message {
    pub fn system(content: String) -> Self {
        Self {
            role: Role::System,
            content,
            timestamp: default_timestamp(),
        }
    }

    pub fn user(content: String) -> Self {
        Self {
            role: Role::User,
            content,
            timestamp: default_timestamp(),
        }
    }

    pub fn assistant(content: String) -> Self {
        Self {
            role: Role::Assistant,
            content,
            timestamp: default_timestamp(),
        }
    }
}

/// A conversation with context window management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    messages: VecDeque<Message>,
    max_messages: usize,
    max_tokens: Option<usize>,
}

impl Conversation {
    pub fn new(system_prompt: Option<String>) -> Self {
        let mut messages = VecDeque::new();
        if let Some(prompt) = system_prompt {
            messages.push_back(Message::system(prompt));
        }

        Self {
            id: Uuid::new_v4().to_string(),
            messages,
            max_messages: 20, // Keep last 20 messages by default
            max_tokens: Some(8000), // Rough token limit
        }
    }

    pub fn with_id(id: String, system_prompt: Option<String>) -> Self {
        let mut conv = Self::new(system_prompt);
        conv.id = id;
        conv
    }

    pub fn with_max_messages(mut self, max_messages: usize) -> Self {
        self.max_messages = max_messages;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: Option<usize>) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Add a message to the conversation.
    pub fn add_message(&mut self, message: Message) {
        self.messages.push_back(message);
        self.trim_context();
    }

    /// Add a user message.
    pub fn add_user_message(&mut self, content: String) {
        self.add_message(Message::user(content));
    }

    /// Add an assistant message.
    pub fn add_assistant_message(&mut self, content: String) {
        self.add_message(Message::assistant(content));
    }

    /// Get all messages in the conversation.
    pub fn messages(&self) -> Vec<Message> {
        self.messages.iter().cloned().collect()
    }

    /// Get the number of messages (excluding system prompt).
    pub fn len(&self) -> usize {
        self.messages
            .iter()
            .filter(|m| m.role != Role::System)
            .count()
    }

    /// Check if conversation is empty (no user/assistant messages).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Trim the conversation to fit within context window.
    fn trim_context(&mut self) {
        // Always keep system prompt (first message if it exists)
        let has_system = self.messages.front().map_or(false, |m| m.role == Role::System);
        let system_offset = if has_system { 1 } else { 0 };

        // Remove old messages if exceeding max_messages
        while self.messages.len() > self.max_messages + system_offset {
            // Remove from position 1 (after system prompt) if exists, else position 0
            if has_system && self.messages.len() > 1 {
                self.messages.remove(1);
            } else if !has_system && !self.messages.is_empty() {
                self.messages.pop_front();
            } else {
                break;
            }
        }

        // TODO: Token-based trimming (requires tokenizer)
        // For now, we use rough character-based heuristic (4 chars ≈ 1 token)
        if let Some(max_tokens) = self.max_tokens {
            let mut total_chars = 0;
            let mut keep_count = 0;

            // Count from the end (most recent messages)
            for msg in self.messages.iter().rev() {
                total_chars += msg.content.len();
                keep_count += 1;

                // Break if we exceed token budget (rough estimate)
                if total_chars / 4 > max_tokens {
                    keep_count -= 1;
                    break;
                }
            }

            // Keep system message even if it exceeds budget
            if has_system {
                keep_count = keep_count.max(1);
            }

            // Remove old messages to fit token budget
            let to_remove = self.messages.len().saturating_sub(keep_count);
            for _ in 0..to_remove {
                if has_system && self.messages.len() > 1 {
                    self.messages.remove(1);
                } else if !has_system && !self.messages.is_empty() {
                    self.messages.pop_front();
                } else {
                    break;
                }
            }
        }
    }

    /// Format conversation for LLM prompt (generic format).
    pub fn to_prompt(&self) -> String {
        self.messages
            .iter()
            .map(|msg| match msg.role {
                Role::System => format!("System: {}\n", msg.content),
                Role::User => format!("User: {}\n", msg.content),
                Role::Assistant => format!("Assistant: {}\n", msg.content),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Clear all messages except system prompt.
    pub fn clear(&mut self) {
        let system_msg = self.messages.front().cloned().filter(|m| m.role == Role::System);
        self.messages.clear();
        if let Some(msg) = system_msg {
            self.messages.push_back(msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_creation() {
        let conv = Conversation::new(Some("You are a helpful assistant.".to_string()));
        assert_eq!(conv.messages.len(), 1);
        assert_eq!(conv.messages[0].role, Role::System);
        assert!(conv.is_empty()); // No user/assistant messages yet
    }

    #[test]
    fn test_add_messages() {
        let mut conv = Conversation::new(None);
        conv.add_user_message("Hello".to_string());
        conv.add_assistant_message("Hi there!".to_string());
        
        assert_eq!(conv.len(), 2);
        assert!(!conv.is_empty());
    }

    #[test]
    fn test_trim_by_message_count() {
        let mut conv = Conversation::new(None).with_max_messages(3);
        
        for i in 0..5 {
            conv.add_user_message(format!("Message {}", i));
        }

        assert_eq!(conv.messages.len(), 3); // Only last 3 messages
    }

    #[test]
    fn test_trim_preserves_system_prompt() {
        let mut conv = Conversation::new(Some("System prompt".to_string())).with_max_messages(2);
        
        conv.add_user_message("User 1".to_string());
        conv.add_assistant_message("Assistant 1".to_string());
        conv.add_user_message("User 2".to_string());
        
        // Should have: system + last 2 messages
        assert_eq!(conv.messages.len(), 3);
        assert_eq!(conv.messages[0].role, Role::System);
    }

    #[test]
    fn test_to_prompt() {
        let mut conv = Conversation::new(Some("Be helpful".to_string()));
        conv.add_user_message("Hello".to_string());
        conv.add_assistant_message("Hi!".to_string());
        
        let prompt = conv.to_prompt();
        assert!(prompt.contains("System: Be helpful"));
        assert!(prompt.contains("User: Hello"));
        assert!(prompt.contains("Assistant: Hi!"));
    }

    #[test]
    fn test_clear_conversation() {
        let mut conv = Conversation::new(Some("System".to_string()));
        conv.add_user_message("Test".to_string());
        
        assert_eq!(conv.len(), 1);
        conv.clear();
        assert_eq!(conv.len(), 0);
        assert_eq!(conv.messages.len(), 1); // System prompt still there
        assert_eq!(conv.messages[0].role, Role::System);
    }
}
