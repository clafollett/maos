# MAOS Architecture Diagrams

## System Overview

```mermaid
graph TB
    subgraph "User Interface Layer"
        U[User] --> CC[Claude Code]
        CC --> CP[Claude Primary Agent]
    end
    
    subgraph "MAOS Core"
        CP --> MO[MAOS Orchestrator]
        MO --> AM[Agent Manager]
        MO --> WM[Worktree Manager]
        MO --> CS[Coordination Service]
        MO --> SH[Security Hooks<br/>Python]
    end
    
    subgraph "Execution Layer"
        AM --> A1[Agent 1<br/>Backend Engineer]
        AM --> A2[Agent 2<br/>Frontend Engineer]
        AM --> A3[Agent 3<br/>QA Engineer]
        AM --> AN[Agent N<br/>...]
        
        WM --> W1[Worktree 1]
        WM --> W2[Worktree 2]
        WM --> W3[Worktree 3]
        WM --> WN[Worktree N]
        
        A1 -.-> W1
        A2 -.-> W2
        A3 -.-> W3
        AN -.-> WN
    end
    
    subgraph "Shared Resources"
        CS --> SC[Shared Context]
        CS --> TR[Task Registry]
        CS --> EQ[Event Queue]
        CS --> DG[Dependency Graph]
        
        SH --> SP[Security Policies]
        SH --> AL[Audit Logs]
    end
    
    style MO fill:#f9f,stroke:#333,stroke-width:4px
    style CS fill:#9ff,stroke:#333,stroke-width:2px
    style SH fill:#ff9,stroke:#333,stroke-width:2px
```

## Component Interaction Flow

```mermaid
sequenceDiagram
    participant U as User
    participant CC as Claude Code
    participant MO as MAOS Orchestrator
    participant AM as Agent Manager
    participant WM as Worktree Manager
    participant CS as Coordination Service
    participant A1 as Agent 1
    participant A2 as Agent 2
    
    U->>CC: Request: "Build authentication system"
    CC->>MO: Parse and plan execution
    
    MO->>MO: Create phase-based plan
    
    loop For each phase
        MO->>AM: Spawn required agents
        AM->>WM: Create worktrees
        WM-->>AM: Worktree info
        AM->>A1: Launch agent with context
        AM->>A2: Launch agent with context
        
        A1->>CS: Register and get tasks
        A2->>CS: Register and get tasks
        
        par Agent 1 Work
            A1->>CS: Get shared context
            A1->>A1: Perform work
            A1->>CS: Update context
            A1->>CS: Emit completion event
        and Agent 2 Work
            A2->>CS: Wait for dependencies
            A2->>A2: Perform work
            A2->>CS: Update context
            A2->>CS: Emit completion event
        end
        
        CS-->>MO: Phase complete
        MO->>MO: Evaluate and replan
    end
    
    MO-->>CC: Execution complete
    CC-->>U: Results delivered
```

## Security Architecture

```mermaid
graph LR
    subgraph "Request Flow"
        R[Request] --> IV[Input Validation]
        IV --> TU[Tool Use]
        TU --> RM[Runtime Monitor]
        RM --> AU[Audit]
    end
    
    subgraph "Hook System"
        IV --> PRE[PreTool Hooks<br/>Python]
        TU --> POST[PostTool Hooks<br/>Python]
        
        PRE --> PE[Policy Engine]
        PE --> PC[Policy Cache]
        PE --> RV[Rule Validator]
        
        POST --> AL[Audit Logger]
        POST --> AM[Anomaly Monitor]
    end
    
    subgraph "Security Layers"
        L1[Layer 1: Input Filtering]
        L2[Layer 2: Operation Validation]
        L3[Layer 3: Runtime Monitoring]
        L4[Layer 4: OS Protection]
        
        L1 --> L2
        L2 --> L3
        L3 --> L4
    end
    
    PRE -.-> L1
    PRE -.-> L2
    POST -.-> L3
    
    style PE fill:#f99,stroke:#333,stroke-width:2px
    style L4 fill:#9f9,stroke:#333,stroke-width:2px
```

## Worktree Management

```mermaid
graph TB
    subgraph "Git Repository"
        GR[Main Git Repo<br/>.git/]
        MB[main branch]
        FB1[feature/auth]
        FB2[feature/api]
        FB3[feature/ui]
    end
    
    subgraph "Worktree Structure"
        ROOT[Project Root]
        ROOT --> MAIN[main/<br/>Primary Worktree]
        ROOT --> WT[worktrees/]
        
        WT --> W1[architect-issue-42/<br/>Branch: architect/issue-42/design]
        WT --> W2[backend-issue-42/<br/>Branch: backend/issue-42/impl]
        WT --> W3[frontend-issue-42/<br/>Branch: frontend/issue-42/ui]
        
        W1 --> AM1[.agent-metadata.json]
        W2 --> AM2[.agent-metadata.json]
        W3 --> AM3[.agent-metadata.json]
    end
    
    subgraph "Agent Assignment"
        A1[Architect Agent] -.-> W1
        A2[Backend Agent] -.-> W2
        A3[Frontend Agent] -.-> W3
    end
    
    GR -.-> MAIN
    FB1 -.-> W1
    FB2 -.-> W2
    FB3 -.-> W3
    
    style GR fill:#ff9,stroke:#333,stroke-width:2px
    style WT fill:#9ff,stroke:#333,stroke-width:2px
```

## Coordination Patterns

```mermaid
graph LR
    subgraph "Task Coordination"
        TC[Task Coordinator]
        TR[Task Registry]
        TQ[Task Queue]
        
        TC --> TR
        TC --> TQ
        
        A1[Agent 1] --> TC
        A2[Agent 2] --> TC
        A3[Agent 3] --> TC
    end
    
    subgraph "Context Sharing"
        CC[Context Coordinator]
        SC[Shared Context]
        CI[Context Index]
        
        CC --> SC
        CC --> CI
        
        A1 --> CC
        A2 --> CC
        A3 --> CC
    end
    
    subgraph "Event System"
        EC[Event Coordinator]
        EQ[Event Queue]
        ES[Event Subscribers]
        
        EC --> EQ
        EC --> ES
        
        A1 --> EC
        A2 --> EC
        A3 --> EC
    end
    
    subgraph "Dependencies"
        DC[Dependency Coordinator]
        DG[Dependency Graph]
        DR[Dependency Resolver]
        
        DC --> DG
        DC --> DR
        
        TC -.-> DC
        EC -.-> DC
    end
    
    style TC fill:#f9f,stroke:#333,stroke-width:2px
    style CC fill:#9ff,stroke:#333,stroke-width:2px
    style EC fill:#ff9,stroke:#333,stroke-width:2px
    style DC fill:#9f9,stroke:#333,stroke-width:2px
```

## Data Flow Architecture

```mermaid
graph TB
    subgraph "Input Processing"
        UI[User Input] --> PA[Prompt Analysis]
        PA --> PP[Phase Planning]
        PP --> TP[Task Planning]
    end
    
    subgraph "Execution Flow"
        TP --> AS[Agent Selection]
        AS --> WC[Worktree Creation]
        WC --> AE[Agent Execution]
        
        AE --> DO[Data Output]
        DO --> SC[Shared Context]
        
        AE --> EE[Event Emission]
        EE --> EQ[Event Queue]
        
        AE --> TU[Task Update]
        TU --> TR[Task Registry]
    end
    
    subgraph "Monitoring Flow"
        AE --> RM[Resource Monitor]
        RM --> MD[Metrics Database]
        
        AE --> AL[Audit Logger]
        AL --> ALF[Audit Log Files]
        
        AE --> HM[Health Monitor]
        HM --> HS[Health Status]
    end
    
    subgraph "Completion Flow"
        DO --> RC[Result Compilation]
        EE --> RC
        TU --> RC
        
        RC --> FR[Final Results]
        FR --> UR[User Response]
    end
    
    style PA fill:#f9f,stroke:#333,stroke-width:2px
    style AE fill:#9ff,stroke:#333,stroke-width:2px
    style RC fill:#9f9,stroke:#333,stroke-width:2px
```

## Performance Architecture

```mermaid
graph LR
    subgraph "Fast Path Components"
        FP[Fast Path]
        RH[Python Hooks<br/>~5-20ms]
        MC[Memory Cache<br/>~0.1ms]
        ZC[Zero-Copy JSON<br/>~1ms]
        
        FP --> RH
        FP --> MC
        FP --> ZC
    end
    
    subgraph "Optimization Layers"
        PH[Parallel Hooks]
        BP[Batch Processing]
        LZ[Lazy Loading]
        PC[Pattern Caching]
        
        RH --> PH
        RH --> PC
        ZC --> BP
        MC --> LZ
    end
    
    subgraph "Resource Management"
        RP[Resource Pooling]
        CP[Connection Pooling]
        TP[Thread Pooling]
        MP[Memory Pooling]
        
        PH --> TP
        BP --> RP
        LZ --> MP
        PC --> CP
    end
    
    subgraph "Monitoring"
        PM[Performance Metrics]
        RT[Response Time]
        TH[Throughput]
        RU[Resource Usage]
        
        RP --> PM
        PM --> RT
        PM --> TH
        PM --> RU
    end
    
    style FP fill:#9f9,stroke:#333,stroke-width:4px
    style RH fill:#9ff,stroke:#333,stroke-width:2px
    style PM fill:#f99,stroke:#333,stroke-width:2px
```

## Deployment Architecture

```mermaid
graph TB
    subgraph "Development Environment"
        DEV[Developer Machine]
        DEV --> LC[Local Claude Code]
        LC --> LM[Local MAOS]
        LM --> LG[Local Git Repos]
    end
    
    subgraph "CI/CD Pipeline"
        GH[GitHub/GitLab]
        GH --> CI[CI Pipeline]
        CI --> UT[Unit Tests]
        CI --> IT[Integration Tests]
        CI --> ST[Security Tests]
        CI --> BD[Build & Package]
    end
    
    subgraph "Distribution"
        BD --> CR[Cargo Registry]
        BD --> BR[Binary Releases]
        BD --> DI[Docker Images]
        
        CR --> UI[User Install<br/>cargo install maos]
        BR --> UD[User Download]
        DI --> UC[User Container]
    end
    
    subgraph "Production Usage"
        UI --> PM[Production MAOS]
        UD --> PM
        UC --> PM
        
        PM --> PG[Production Git]
        PM --> PS[Production Storage]
        PM --> PL[Production Logs]
    end
    
    style DEV fill:#9ff,stroke:#333,stroke-width:2px
    style CI fill:#f9f,stroke:#333,stroke-width:2px
    style PM fill:#9f9,stroke:#333,stroke-width:2px
```

## Failure Recovery Architecture

```mermaid
graph TB
    subgraph "Failure Detection"
        HB[Heartbeat Monitor]
        TC[Timeout Checker]
        EC[Error Counter]
        AC[Anomaly Checker]
        
        HB --> FD[Failure Detector]
        TC --> FD
        EC --> FD
        AC --> FD
    end
    
    subgraph "Recovery Strategies"
        FD --> RS[Recovery Selector]
        
        RS --> RR[Restart Recovery]
        RS --> FR[Failover Recovery]
        RS --> GD[Graceful Degradation]
        RS --> CB[Circuit Breaker]
    end
    
    subgraph "Recovery Actions"
        RR --> RA[Restart Agent]
        FR --> SA[Switch Agent]
        GD --> RM[Reduce Mode]
        CB --> PH[Pause & Hold]
        
        RA --> HC[Health Check]
        SA --> HC
        RM --> HC
        PH --> HC
    end
    
    subgraph "State Recovery"
        HC --> SR[State Recovery]
        SR --> WS[Worktree State]
        SR --> TS[Task State]
        SR --> CS[Context State]
        
        WS --> RC[Resume/Continue]
        TS --> RC
        CS --> RC
    end
    
    style FD fill:#f99,stroke:#333,stroke-width:2px
    style RS fill:#ff9,stroke:#333,stroke-width:2px
    style SR fill:#9f9,stroke:#333,stroke-width:2px
```

## Summary

These architectural diagrams illustrate the key components and interactions within the MAOS system:

1. **System Overview**: Shows the layered architecture from user interface to shared resources
2. **Component Interaction**: Details the sequence of operations for multi-agent workflows
3. **Security Architecture**: Illustrates the multi-layered security approach
4. **Worktree Management**: Demonstrates git worktree organization and agent assignment
5. **Coordination Patterns**: Shows how agents coordinate through various services
6. **Data Flow**: Traces data movement through the system
7. **Performance Architecture**: Highlights optimization strategies
8. **Deployment Architecture**: Shows development to production pipeline
9. **Failure Recovery**: Illustrates resilience and recovery mechanisms

Each diagram focuses on a specific aspect while maintaining consistency with the overall architecture, providing a comprehensive view of the MAOS multi-agent orchestration system.