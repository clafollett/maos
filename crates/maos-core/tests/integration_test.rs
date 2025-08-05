use chrono::Utc;
use maos_core::{AgentId, AgentInfo, AgentStatus, Session, SessionId, SessionStatus, ToolCall};
use std::path::PathBuf;

#[test]
fn test_full_type_integration() {
    // Create a session
    let session = Session {
        id: SessionId::generate(),
        created_at: Utc::now(),
        last_activity: Utc::now(),
        status: SessionStatus::Active,
        workspace_root: PathBuf::from("/tmp/maos-test"),
        active_agents: vec!["agent_1".to_string()],
    };

    // Create an agent
    let agent = AgentInfo {
        id: AgentId::generate(),
        agent_type: "test-agent".to_string(),
        session_id: session.id.clone(),
        workspace_path: PathBuf::from("/tmp/maos-test/agent-workspace"),
        status: AgentStatus::Active,
        created_at: Utc::now(),
        last_activity: Utc::now(),
    };

    // Create a tool call
    let tool_call = ToolCall {
        id: "call_123".to_string(),
        tool_name: "Bash".to_string(),
        parameters: serde_json::json!({ "command": "echo test" }),
        timestamp: Utc::now(),
        session_id: Some(session.id.clone()),
        agent_id: Some(agent.id.clone()),
    };

    // Verify everything works together
    assert!(session.id.is_valid());
    assert!(agent.id.is_valid());
    assert_eq!(tool_call.session_id.as_ref().unwrap(), &session.id);
}

#[test]
fn test_serialization_roundtrip() {
    let session_id = SessionId::generate();

    // Serialize and deserialize
    let json = serde_json::to_string(&session_id).unwrap();
    let deserialized: SessionId = serde_json::from_str(&json).unwrap();

    assert_eq!(session_id, deserialized);
    assert!(deserialized.is_valid());
}
