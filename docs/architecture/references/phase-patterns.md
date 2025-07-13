# MAOS Phase Patterns Reference

## Overview

This document describes common orchestration phase patterns for organizing multi-agent work in MAOS. Phase patterns provide proven approaches for structuring complex projects into manageable, coordinated stages.

## Phase Pattern Categories

### Sequential Patterns

#### 1. Waterfall Pattern
**Use Case**: Projects with clear dependencies and stable requirements

```json
{
  "objective": "Build web application",
  "strategy": "sequential",
  "phases": [
    {
      "name": "Requirements Analysis",
      "agents": [{"role": "analyst", "task": "Analyze business requirements"}]
    },
    {
      "name": "Architecture Design", 
      "agents": [{"role": "solution_architect", "task": "Design system architecture"}],
      "dependencies": ["Requirements Analysis"]
    },
    {
      "name": "Implementation",
      "agents": [{"role": "backend_engineer", "task": "Implement application"}],
      "dependencies": ["Architecture Design"]
    },
    {
      "name": "Testing",
      "agents": [{"role": "qa", "task": "Test application"}],
      "dependencies": ["Implementation"]
    }
  ]
}
```

**Characteristics**:
- Clear phase boundaries
- Minimal overlap between phases
- Strong dependency enforcement
- Predictable timeline

#### 2. Progressive Refinement Pattern
**Use Case**: Complex systems requiring iterative design refinement

```json
{
  "objective": "Design enterprise platform",
  "strategy": "sequential", 
  "phases": [
    {
      "name": "High-Level Design",
      "agents": [{"role": "solution_architect", "task": "Create solution overview"}]
    },
    {
      "name": "Detailed Architecture",
      "agents": [
        {"role": "application_architect", "task": "Design application structure"},
        {"role": "data_architect", "task": "Design data architecture"}
      ],
      "dependencies": ["High-Level Design"]
    },
    {
      "name": "Technical Specifications",
      "agents": [
        {"role": "api_architect", "task": "Design API specifications"},
        {"role": "security_architect", "task": "Design security controls"}
      ],
      "dependencies": ["Detailed Architecture"]
    },
    {
      "name": "Implementation Planning",
      "agents": [
        {"role": "backend_engineer", "task": "Create implementation plan"},
        {"role": "devops", "task": "Plan deployment strategy"}
      ],
      "dependencies": ["Technical Specifications"]
    }
  ]
}
```

### Parallel Patterns

#### 3. Concurrent Specialization Pattern
**Use Case**: Independent workstreams that can proceed simultaneously

```json
{
  "objective": "Develop microservices platform", 
  "strategy": "parallel",
  "phases": [
    {
      "name": "Parallel Design",
      "agents": [
        {"role": "solution_architect", "task": "Design platform architecture"},
        {"role": "data_architect", "task": "Design data strategy"},
        {"role": "api_architect", "task": "Design API standards"},
        {"role": "security_architect", "task": "Design security framework"}
      ]
    },
    {
      "name": "Service Implementation",
      "agents": [
        {"role": "backend_engineer", "instance_suffix": "auth", "task": "Implement auth service"},
        {"role": "backend_engineer", "instance_suffix": "api", "task": "Implement API gateway"},
        {"role": "backend_engineer", "instance_suffix": "data", "task": "Implement data services"}
      ],
      "dependencies": ["Parallel Design"]
    }
  ]
}
```

#### 4. Research and Development Pattern
**Use Case**: Projects requiring investigation and prototyping

```json
{
  "objective": "Evaluate ML platform options",
  "strategy": "parallel",
  "phases": [
    {
      "name": "Technology Research",
      "agents": [
        {"role": "researcher", "instance_suffix": "aws", "task": "Research AWS ML services"},
        {"role": "researcher", "instance_suffix": "azure", "task": "Research Azure ML services"},
        {"role": "researcher", "instance_suffix": "gcp", "task": "Research Google ML services"}
      ]
    },
    {
      "name": "Proof of Concept",
      "agents": [
        {"role": "data_scientist", "task": "Build ML pipeline prototype"},
        {"role": "backend_engineer", "task": "Implement integration prototype"}
      ],
      "dependencies": ["Technology Research"]
    },
    {
      "name": "Evaluation",
      "agents": [
        {"role": "analyst", "task": "Compare solutions and recommend"}
      ],
      "dependencies": ["Proof of Concept"]
    }
  ]
}
```

### Hybrid Patterns

#### 5. Pipeline Pattern
**Use Case**: Data processing or staged transformation workflows

```json
{
  "objective": "Build data analytics pipeline",
  "strategy": "pipeline",
  "phases": [
    {
      "name": "Data Ingestion Design",
      "agents": [{"role": "data_architect", "task": "Design data ingestion"}]
    },
    {
      "name": "Processing Pipeline",
      "agents": [
        {"role": "data_scientist", "task": "Design data transformations"},
        {"role": "backend_engineer", "task": "Implement processing pipeline"}
      ],
      "dependencies": ["Data Ingestion Design"],
      "execution": "parallel"
    },
    {
      "name": "Analytics Layer", 
      "agents": [
        {"role": "data_scientist", "task": "Build analytics models"},
        {"role": "backend_engineer", "task": "Implement API layer"}
      ],
      "dependencies": ["Processing Pipeline"],
      "execution": "parallel"
    },
    {
      "name": "Deployment",
      "agents": [
        {"role": "devops", "task": "Deploy pipeline"},
        {"role": "qa", "task": "Test end-to-end pipeline"}
      ],
      "dependencies": ["Analytics Layer"]
    }
  ]
}
```

#### 6. Agile Sprint Pattern
**Use Case**: Iterative development with regular review cycles

```json
{
  "objective": "Develop customer portal",
  "strategy": "adaptive",
  "phases": [
    {
      "name": "Sprint Planning",
      "agents": [
        {"role": "pm", "task": "Plan sprint objectives"},
        {"role": "analyst", "task": "Refine user stories"}
      ]
    },
    {
      "name": "Design Sprint",
      "agents": [
        {"role": "ux_designer", "task": "Design user interfaces"},
        {"role": "application_architect", "task": "Design technical approach"}
      ],
      "dependencies": ["Sprint Planning"],
      "execution": "parallel"
    },
    {
      "name": "Development Sprint",
      "agents": [
        {"role": "frontend_engineer", "task": "Implement UI"},
        {"role": "backend_engineer", "task": "Implement API"}
      ],
      "dependencies": ["Design Sprint"],
      "execution": "parallel"
    },
    {
      "name": "Review & Testing",
      "agents": [
        {"role": "qa", "task": "Test sprint deliverables"},
        {"role": "reviewer", "task": "Review code and design"}
      ],
      "dependencies": ["Development Sprint"]
    }
  ]
}
```

### Specialized Patterns

#### 7. Security-First Pattern
**Use Case**: Security-critical applications requiring threat modeling

```json
{
  "objective": "Build secure financial application",
  "strategy": "sequential",
  "phases": [
    {
      "name": "Security Requirements",
      "agents": [
        {"role": "security_architect", "task": "Define security requirements"},
        {"role": "analyst", "task": "Analyze compliance needs"}
      ]
    },
    {
      "name": "Threat Modeling", 
      "agents": [
        {"role": "security_architect", "task": "Perform threat modeling"},
        {"role": "solution_architect", "task": "Design secure architecture"}
      ],
      "dependencies": ["Security Requirements"]
    },
    {
      "name": "Secure Implementation",
      "agents": [
        {"role": "backend_engineer", "task": "Implement with security controls"},
        {"role": "security", "task": "Review security implementation"}
      ],
      "dependencies": ["Threat Modeling"]
    },
    {
      "name": "Security Testing",
      "agents": [
        {"role": "security", "task": "Perform security testing"},
        {"role": "qa", "task": "Functional testing"}
      ],
      "dependencies": ["Secure Implementation"]
    }
  ]
}
```

#### 8. API-First Pattern
**Use Case**: Service development with API contracts as primary interface

```json
{
  "objective": "Build microservice with API-first approach",
  "strategy": "sequential",
  "phases": [
    {
      "name": "API Design",
      "agents": [
        {"role": "api_architect", "task": "Design API specification"},
        {"role": "analyst", "task": "Define API requirements"}
      ]
    },
    {
      "name": "Contract Validation",
      "agents": [
        {"role": "backend_engineer", "task": "Create API mock"},
        {"role": "qa", "task": "Test API contract"}
      ],
      "dependencies": ["API Design"]
    },
    {
      "name": "Implementation",
      "agents": [
        {"role": "backend_engineer", "task": "Implement API service"},
        {"role": "data_architect", "task": "Implement data layer"}
      ],
      "dependencies": ["Contract Validation"],
      "execution": "parallel"
    },
    {
      "name": "Integration Testing",
      "agents": [
        {"role": "qa", "task": "Test API integration"},
        {"role": "backend_engineer", "task": "Performance testing"}
      ],
      "dependencies": ["Implementation"]
    }
  ]
}
```

## Phase Pattern Selection Guide

### By Project Characteristics

#### Simple, Well-Defined Projects
- **Waterfall Pattern**: Clear requirements, stable scope
- **Progressive Refinement**: Complex but predictable

#### Complex, Dynamic Projects  
- **Agile Sprint Pattern**: Changing requirements, iterative feedback
- **Research and Development**: Uncertain technology choices

#### Performance-Critical Projects
- **Pipeline Pattern**: Data processing, transformation workflows
- **Concurrent Specialization**: Independent optimization efforts

#### Security/Compliance-Critical Projects
- **Security-First Pattern**: Threat modeling, compliance requirements
- **Progressive Refinement**: Layered security controls

### By Team Structure

#### Small Teams (2-4 agents)
- **Waterfall Pattern**: Sequential execution with clear handoffs
- **Pipeline Pattern**: Staged progression through phases

#### Medium Teams (5-8 agents)
- **Concurrent Specialization**: Parallel workstreams
- **Agile Sprint Pattern**: Collaborative iteration

#### Large Teams (9+ agents)
- **Research and Development**: Multiple parallel investigations
- **Hybrid patterns**: Combine sequential and parallel phases

## Phase Pattern Implementation

### Phase Definition Structure

```json
{
  "name": "Phase Name",
  "description": "What this phase accomplishes",
  "agents": [
    {
      "role": "role_name",
      "instance_suffix": "optional_suffix", 
      "task": "Specific task description",
      "model": "optional_model_override"
    }
  ],
  "dependencies": ["prerequisite_phase_names"],
  "execution": "sequential|parallel|adaptive",
  "timeout": "optional_phase_timeout",
  "success_criteria": "How to determine phase completion"
}
```

### Dependency Management

#### Simple Dependencies
```json
"dependencies": ["Requirements Analysis"]
```

#### Multiple Dependencies
```json
"dependencies": ["Architecture Design", "Security Review"]
```

#### Conditional Dependencies
```json
"dependencies": {
  "required": ["Architecture Design"],
  "optional": ["Performance Analysis"],
  "conditions": {
    "Performance Analysis": "if high_performance_required"
  }
}
```

### Execution Strategies

#### Sequential (Default)
- Agents run one after another
- Clear handoffs between agents
- Predictable resource usage

#### Parallel
- All agents run simultaneously  
- Faster execution
- Higher resource requirements

#### Adaptive
- Dynamic scheduling based on dependencies
- Optimal resource utilization
- More complex coordination

## Best Practices

### Phase Design
1. **Clear Objectives**: Each phase should have specific, measurable goals
2. **Manageable Scope**: Limit phase complexity for better coordination
3. **Clean Dependencies**: Minimize cross-phase dependencies
4. **Resource Planning**: Consider memory and timeout requirements

### Agent Coordination
1. **Role Specialization**: Match agent expertise to phase requirements
2. **Instance Management**: Use instance suffixes for parallel similar roles
3. **Communication Planning**: Ensure agents can coordinate effectively
4. **Handoff Protocols**: Define clear deliverables between phases

### Error Handling
1. **Phase Rollback**: Plan for phase failure scenarios
2. **Dependency Handling**: Graceful handling of failed dependencies
3. **Recovery Strategies**: Options for resuming from partial completion
4. **Escalation Paths**: When to involve human oversight

### Performance Optimization
1. **Parallel Opportunities**: Identify independent work streams
2. **Resource Balancing**: Distribute computational load across phases  
3. **Critical Path**: Optimize the longest dependency chain
4. **Checkpointing**: Regular state saves for recovery

## Anti-Patterns to Avoid

### Over-Parallelization
- **Problem**: Too many concurrent agents causing resource contention
- **Solution**: Balance parallelism with available resources

### Tight Coupling
- **Problem**: Phases with too many inter-dependencies
- **Solution**: Redesign phases for better independence

### Monolithic Phases
- **Problem**: Phases that are too large or complex
- **Solution**: Break down into smaller, focused phases

### Under-Specified Handoffs
- **Problem**: Unclear deliverables between phases
- **Solution**: Define explicit outputs and success criteria

---
*Last Updated: 2025-07-13*
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollett - LaFollett Labs LLC)*