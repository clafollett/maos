# ADR-03: Session Orchestration and State Management

## Status
Accepted

## Context
MAOS orchestration happens within sessions - logical groupings of agents working together on a specific objective. Sessions need comprehensive state management and recovery capabilities to handle the complex orchestration scenarios revealed by our POC.

### Core Session Requirements
- Track the overall orchestration objective and progress
- Manage agent lifecycles within the session
- Coordinate dependencies and execution order
- Handle failures and recovery at multiple levels
- Provide visibility into session state and progress
- Clean up resources when complete
- Support interrupted session recovery

### POC-Informed Recovery Requirements
During POC development, we discovered that orchestration sessions could be interrupted due to various factors including model timeouts, system crashes, network issues, and user intervention. Without proper state management and recovery mechanisms, these interruptions resulted in lost work and required complete restarts.

Key requirements evolved to include:
- Sessions persist across MAOS restarts with full state recovery
- Multiple sessions can run concurrently without interference  
- Sessions can be paused and resumed at agent, phase, or session level
- Complete session history for debugging and recovery
- Integration with Claude CLI `--resume` functionality
- Hierarchical recovery strategies for different interruption types

## Decision
We will implement comprehensive session management with state tracking, persistence, and lifecycle control.

### Architectural Layering

This ADR focuses on high-level session orchestration and coordination, while delegating process-level management to ADR-08:

- **ADR-03 handles**: Session planning, agent coordination, dependency tracking, execution strategies, and recovery
- **ADR-08 handles**: Process spawning, resource management, health monitoring, and lifecycle states
- **Relationship**: ADR-03 orchestrates sessions by coordinating with ADR-08's process management infrastructure

### Unified State Model

This ADR uses the unified state model that provides consistent state definitions across all orchestration layers:

```rust
// Session-level orchestration states
pub enum SessionState {
    Created,      // Planning phase
    Running,      // Active execution
    Paused,       // Temporarily suspended
    Completed,    // Successfully finished
    Failed,       // Terminated due to errors
    Cancelled,    // User-initiated termination
}

// Agent execution states (shared across ADR-03, ADR-04, ADR-08)
pub enum AgentExecutionState {
    Pending,      // Waiting to start
    Running,      // Currently executing
    Completed,    // Successfully finished
    Failed,       // Terminated due to error
    Resumable,    // Interrupted but can resume (for recovery)
    Cancelled,    // Explicitly stopped
}

// Phase states for orchestration
pub enum PhaseState {
    Planned,      // Phase defined but not started
    Running,      // Phase actively executing
    Completed,    // Phase successfully finished
    Failed,       // Phase failed
    Skipped,      // Phase skipped due to conditions
}

// Execution strategies
pub enum ExecutionStrategy {
    Parallel,     // All agents run concurrently
    Sequential,   // Agents run one after another
    Adaptive,     // Dynamic scheduling based on dependencies
    Pipeline,     // Agents in stages with handoffs
}

pub struct Session {
    pub id: String,
    pub workspace_hash: String,
    pub objective: String,
    pub strategy: ExecutionStrategy,
    pub state: SessionState,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub agent_count: usize,
    pub completed_agents: usize,
    pub failed_agents: usize,
    pub metadata: HashMap<String, Value>,
}
```

### Session Persistence

Session tracking is managed through the project database schema documented in the [Storage Schema Reference](../references/storage-schema.md#project-database-schema). Key tables include:

- **sessions**: Tracks orchestration sessions with state, strategy, and progress
- **session_agents**: Manages agent assignments, tasks, and dependencies
- **session_events**: Provides audit trail for session lifecycle events

### Session Lifecycle Manager

```rust
pub struct SessionManager {
    db: SqlitePool,
    process_manager: Arc<ProcessManager>,
    logger: Arc<SessionLogger>,
}

### Session Lifecycle Management

The SessionManager coordinates session creation, execution, and lifecycle transitions:

- **create_session()**: Initializes session with planning phase, agent registration, and database persistence
- **start_session()**: Transitions to running state and initiates agents based on execution strategy
- **pause_session()**: Suspends session and delegates agent pausing to ProcessManager
- **resume_session()**: Restores session state and resumes or starts pending agents

Session strategies determine agent coordination:
- **Parallel**: All agents execute concurrently
- **Sequential**: Agents execute in defined order
- **Adaptive**: Dynamic scheduling based on dependency resolution
- **Pipeline**: Staged execution with handoffs between phases
```

### Agent Coordination

The session manager coordinates agents through dependency tracking and state management:

- **Dependency Resolution**: Checks completion status of prerequisite agents before starting dependent agents
- **Agent Lifecycle Events**: Handles completion notifications and updates session progress counters
- **Strategy-Based Execution**: Adapts agent startup based on the session's execution strategy
- **Progress Tracking**: Maintains session-level counters for completed and failed agents

### Session Monitoring

Real-time session monitoring provides visibility into orchestration progress:

- **Session Status**: Current state, agent distribution, and progress metrics
- **Role Distribution**: Statistics on agent roles and their execution states
- **Progress Estimation**: Calculated completion estimates based on agent states
- **Event Streaming**: Real-time updates on session and agent state changes

### Comprehensive State Management and Recovery

#### State Persistence Strategy
We maintain persistent state for all orchestration sessions including:
- **Current phase information**: Active phase, completed phases, planned phases
- **Agent execution state**: Running agents, completed agents, pending agents  
- **Orchestrator plan state**: Adaptive plan evolution, phase decisions, dependencies
- **Shared context state**: Phase outputs, summaries, shared documents

#### Hierarchical Resume Support
Support resume at multiple levels:
- **Agent-Level Resume**: Individual agents resume using Claude CLI `--resume` flag
- **Phase-Level Resume**: Resume orchestration from the beginning of an interrupted phase
- **Session-Level Resume**: Resume entire orchestration session from last checkpoint

#### State Checkpointing Strategy
Create state checkpoints at key orchestration points:
- **Phase Boundaries**: Before starting each new phase
- **Agent Completion**: After each agent completes successfully
- **Plan Updates**: When Orchestrator modifies or extends the plan
- **Error Conditions**: Before attempting error recovery

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OrchestrationState {
    session_id: String,
    current_phase: Option<Phase>,
    completed_phases: Vec<Phase>,
    planned_phases: Vec<PlannedPhase>,
    orchestrator_context: OrchestratorContext,
    shared_context: SharedContext,
    last_checkpoint: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Phase {
    phase_id: String,
    phase_name: String,
    state: PhaseState, // Uses unified state model
    agents: Vec<AgentExecution>,
    outputs: Vec<PhaseOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentExecution {
    agent_id: String,
    role: String,
    state: AgentExecutionState, // Uses unified state model
    claude_session_id: Option<String>, // For --resume integration
    start_time: Option<DateTime<Utc>>,
    completion_time: Option<DateTime<Utc>>,
}

### Session Recovery Implementation

**Recovery Process**: 
- Identify interrupted sessions from database state
- Load orchestration state from checkpoint files
- Assess recovery scope based on interruption context
- Execute appropriate recovery strategy

**Recovery Strategies**:
- **Agent Resume**: Use Claude CLI `--resume` for individual agents with saved session IDs
- **Phase Restart**: Restart current phase with preserved context and dependencies
- **Session Resume**: Full session recovery from last checkpoint
- **Graceful Failure**: Mark session as failed when recovery isn't viable

**Checkpoint Management**:
- Structured JSON checkpoints at phase boundaries and key orchestration points
- Automatic checkpoint creation before risky operations
- Human-readable format for debugging and manual inspection

#[derive(Debug, Clone)]
enum RecoveryPlan {
    ResumeAgent(String),
    RestartPhase(String), 
    ResumeSession,
    MarkFailed(String),
}
```

### Session Cleanup

Automated cleanup manages session lifecycle:
- **Configurable Retention**: Cleanup sessions after specified retention period
- **Selective Archiving**: Optional archiving of completed sessions before deletion  
- **Directory Cleanup**: Remove session workspace directories and artifacts
- **Database Cleanup**: Remove session records and related agent data

## Consequences

### Positive
- **Resilient Orchestration**: Sessions survive various types of interruptions with comprehensive recovery
- **Reduced Waste**: No need to restart completed work after interruptions
- **Better User Experience**: Users can confidently start long-running orchestrations
- **Fault Tolerance**: System gracefully handles expected failure modes with intelligent recovery
- **Multiple Execution Strategies**: Parallel, Sequential, Adaptive, and Pipeline orchestration modes
- **Comprehensive Visibility**: Clear progress tracking with role distribution and phase-aware monitoring
- **Development Efficiency**: Faster iteration during development and testing with resume capability
- **Complete Session History**: Full audit trail for debugging and analysis

### Negative
- **Implementation Complexity**: Additional state management and recovery logic across multiple abstraction levels
- **Storage Requirements**: Persistent state storage for all orchestration sessions and checkpoints
- **Performance Overhead**: Checkpointing adds latency to orchestration execution
- **Recovery Edge Cases**: Complex scenarios where automated recovery isn't possible
- **Database Operations**: Ongoing overhead for state tracking and updates

### Mitigation
- Implement incremental state storage to minimize performance impact
- Provide clear user feedback on recovery options and limitations
- Create comprehensive testing for various interruption scenarios
- Design graceful degradation when automated recovery isn't possible
- Automatic cleanup policies for completed sessions
- Efficient state queries with proper indexing

## References
- ADR-02: Hybrid Storage Strategy - Session data persistence
- ADR-07: Orchestration Guardrails and Coordination Protocols - Session coordination requirements
- ADR-08: Agent Lifecycle and Management - Process-level coordination with session management
- ADR-11: Adaptive Phase-Based Orchestration - Phase-aware session planning
- Claude CLI `--resume` functionality documentation
- POC analysis of session interruption patterns
- Workflow orchestration patterns
- State machine design
- Fault tolerance design patterns

---
*Date: 2025-07-13*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollett - LaFollett Labs LLC)*