# Orchestration Guardrails for MAOS

## Problem Statement

Our POC revealed a critical issue: the orchestrator created vague task assignments that led to architects designing comprehensive systems while engineers built basic prototypes that ignored the architectural decisions.

## Key Issues Identified

1. **Communication Breakdown**: Engineers never referenced architectural docs
2. **Vague Task Assignments**: "Build components" without specific requirements
3. **Missing Coordination**: No validation between design and implementation phases
4. **Parallel Execution Risks**: Multiple agents building incompatible systems

## Proposed Solutions

### 1. Mandatory Coordination Phases

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EnhancedExecutionPlan {
    phases: Vec<EnhancedPhase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EnhancedPhase {
    name: String,
    parallel: bool,
    agents: Vec<EnhancedAgentSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EnhancedAgentSpec {
    role: String,
    task: String,
    requirements: Option<AgentRequirements>,
    coordination: Option<CoordinationSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentRequirements {
    must_read: Vec<String>,
    ui_specifications: Option<UiSpec>,
    api_integration: Option<ApiSpec>,
    validation_criteria: Vec<String>,
}

// Example enhanced plan:
let enhanced_plan = EnhancedExecutionPlan {
    phases: vec![
        EnhancedPhase {
            name: "Research and Requirements".to_string(),
            parallel: false,
            agents: vec![/* ... */],
        },
        EnhancedPhase {
            name: "Architecture and Design".to_string(),
            parallel: false,
            agents: vec![/* ... */],
        },
        EnhancedPhase {
            name: "Project Setup and Coordination".to_string(),
            parallel: false,
            agents: vec![
                EnhancedAgentSpec {
                    role: "architect".to_string(),
                    task: "Create implementation guidelines and project structure based on architectural decisions".to_string(),
                    requirements: Some(AgentRequirements {
                        must_read: vec!["shared_context/architectural_decisions.md".to_string()],
                        validation_criteria: vec!["Implementation guide is comprehensive and actionable".to_string()],
                        ..Default::default()
                    }),
                    coordination: Some(CoordinationSpec {
                        creates_for: vec!["engineer".to_string()],
                        ..Default::default()
                    }),
                },
                EnhancedAgentSpec {
                    role: "engineer".to_string(),
                    task: "Review architectural decisions and create implementation plan with specific milestones".to_string(),
                    requirements: Some(AgentRequirements {
                        must_read: vec![
                            "docs/design/system-architecture.md".to_string(),
                            "IMPLEMENTATION_GUIDE.md".to_string(),
                        ],
                        validation_criteria: vec!["Implementation plan confirms architectural compliance".to_string()],
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ],
        },
        EnhancedPhase {
            name: "Core Implementation".to_string(),
            parallel: true,
            agents: vec![/* ... */],
        },
    ],
};
```

### 2. Enhanced Task Specifications

Instead of:
```
"task": "Build Vue components for project management features"
```

Use:
```
"task": "Build Vue components for project management following the wireframes in docs/design/wireframes-project-management.md. Use 56px touch targets for field optimization as specified in docs/design/ui-ux-design-system.md. Implement the microservices architecture with Project Service endpoints from docs/api/api-specification.md"
```

### 3. Compliance Checking Agents

Add specialized agents:
```rust
EnhancedAgentSpec {
    role: "compliance_checker".to_string(),
    task: "Review implementation against architectural decisions and report discrepancies".to_string(),
    requirements: Some(AgentRequirements {
        must_read: vec![
            "docs/design/system-architecture.md".to_string(),
            "project/backend/".to_string(),
            "project/frontend/".to_string(),
        ],
        validation_criteria: vec![
            "Backend follows microservices architecture".to_string(),
            "Frontend uses 56px touch targets".to_string(),
            "API contracts match specification".to_string(),
        ],
        ..Default::default()
    }),
    coordination: Some(CoordinationSpec {
        reports_to: "shared_context/compliance_report.md".to_string(),
        blocks_next_phase: true,
        ..Default::default()
    }),
}
```

### 4. Cross-Agent Communication Protocol

#### Required Shared Context Format:
```markdown
# Agent Summary: [ROLE]_[ID]

## Key Decisions Made
- [Specific architectural/implementation decisions]

## Requirements for Other Agents
- [What other agents MUST implement to be compatible]

## Validation Criteria
- [How other agents can verify compliance]
```

### 5. Implementation Validation Gates

Before proceeding to next phase:
1. **Architecture Compliance Check**: Verify implementation matches design
2. **Integration Test**: Ensure all modules work together
3. **Requirements Validation**: Confirm all user stories are implemented

### 6. Enhanced Agent Prompts

#### For Engineers:
```
CRITICAL INSTRUCTIONS:
1. READ the architectural decisions in docs/design/ BEFORE starting
2. IMPLEMENT the specific UI patterns defined in wireframes
3. FOLLOW the technology stack specified by architects
4. VALIDATE your work against the API specification
5. COORDINATE with other engineers through shared context

Your task: [specific task]
Architecture to follow: docs/design/system-architecture.md
UI specifications: docs/design/ui-ux-design-system.md
API contracts: docs/api/api-specification.md
```

## Implementation Plan

### Phase 1: Update Orchestrator Prompts
- Add specific architecture references to all engineering tasks
- Include compliance checking requirements

### Phase 2: Add Coordination Phases
- Insert "Project Setup" phase after architecture
- Add validation gates between phases

### Phase 3: Create Compliance Agents
- Build agents that verify implementation vs design
- Add cross-module integration testing

### Phase 4: Enhanced Communication
- Standardize shared context format
- Add requirement tracking across agents

## Success Metrics

1. **Architecture Compliance**: 95% implementation matches design
2. **Integration Success**: All modules work together without conflicts
3. **Requirements Coverage**: 100% of architectural decisions implemented
4. **Communication Quality**: Clear handoffs between agent phases

## Risk Mitigation

1. **Fallback Plans**: If compliance fails, spawn correction agents
2. **Validation Points**: Multiple checkpoints to catch issues early
3. **Communication Loops**: Agents must acknowledge receipt of requirements