# MAOS Model Selection Guide

## Overview

This guide provides recommendations for selecting appropriate Claude models for different agent roles and task complexities. Model selection affects both performance and cost, so choosing the right model for each role is important for optimal orchestration.

## Model Characteristics

### Claude 4 Opus (claude-opus-4-20250514)
- **Strengths**: Ultimate reasoning capability, complex analysis, strategic thinking
- **Best For**: Complex architecture decisions, research, critical analysis, enterprise strategy
- **Cost**: Highest
- **Speed**: Moderate

### Claude 4 Sonnet (claude-sonnet-4-20250514)
- **Strengths**: Excellent reasoning and coding, balanced performance
- **Best For**: Development tasks, application architecture, technical implementation
- **Cost**: Premium
- **Speed**: Fast



## Role-Based Model Recommendations

### Meta-Role

#### Orchestrator
- **Recommended**: Claude 4 Opus
- **Reasoning**: Orchestration requires ultimate reasoning for complex strategic planning, adaptive coordination, and multi-agent workflow optimization
- **No Fallback**: Always uses Claude 4 Opus due to critical importance of orchestration decisions

### Architecture Roles

#### Solution Architect
- **Recommended**: Claude 4 Opus
- **Reasoning**: Enterprise solution design requires ultimate reasoning for complex decision-making and strategic analysis
- **Fallback**: Claude 4 Sonnet for cost-conscious deployments

#### Application Architect
- **Recommended**: Claude 4 Sonnet
- **Reasoning**: Benefits from superior reasoning for application design patterns and architectural decisions
- **Fallback**: Claude 4 Sonnet for simple applications

#### Data Architect
- **Recommended**: Claude 4 Opus or Claude 4 Sonnet
- **Reasoning**: Complex data modeling and enterprise data strategy benefit from ultimate reasoning capabilities
- **Fallback**: Claude 4 Sonnet for straightforward data designs

#### API Architect  
- **Recommended**: Claude 4 Sonnet
- **Reasoning**: API design patterns benefit from strong reasoning and coding capabilities
- **Fallback**: Claude 4 Sonnet for simple CRUD APIs

#### Security Architect
- **Recommended**: Claude 4 Opus
- **Reasoning**: Security and threat modeling require the absolute highest level of analysis and comprehensive understanding
- **Fallback**: Claude 4 Sonnet for standard security patterns

### Development Roles

#### Backend Engineer
- **Recommended**: Claude 4 Sonnet
- **Reasoning**: Server-side development benefits from excellent coding and reasoning capabilities
- **Fallback**: Claude 4 Sonnet for simple API implementations

#### Frontend Engineer
- **Recommended**: Claude 4 Sonnet
- **Reasoning**: UI development and client-side logic benefit from strong coding and problem-solving
- **Fallback**: Claude 4 Sonnet for simple component implementations

#### Mobile Engineer
- **Recommended**: Claude 4 Sonnet
- **Reasoning**: Platform-specific development requires strong coding skills and platform knowledge
- **Fallback**: Claude 4 Sonnet for simple mobile features

#### DevOps
- **Recommended**: Claude 4 Sonnet
- **Reasoning**: Good for infrastructure automation and configuration
- **Fallback**: Claude 4 Sonnet for standard deployments

#### Security (Implementation)
- **Recommended**: Claude 4 Sonnet or Claude 4 Opus
- **Reasoning**: Security implementation requires careful attention to detail
- **Fallback**: Claude 4 Sonnet for standard security implementations

### Analysis Roles

#### Researcher
- **Recommended**: Claude 4 Opus
- **Reasoning**: Research requires ultimate reasoning for deep analysis, comparison, and comprehensive investigation
- **Fallback**: Claude 4 Sonnet for focused research tasks

#### Data Scientist
- **Recommended**: Claude 4 Opus or Claude 4 Sonnet
- **Reasoning**: Complex statistical analysis and ML model selection benefit from ultimate reasoning capabilities
- **Fallback**: Claude 4 Sonnet for standard data science tasks

#### Analyst
- **Recommended**: Claude 4 Opus or Claude 4 Sonnet
- **Reasoning**: Complex business analysis benefits from ultimate reasoning and comprehensive understanding
- **Fallback**: Claude 4 Sonnet for straightforward analysis

### Quality & Review Roles

#### QA
- **Recommended**: Claude 4 Sonnet or Claude 4 Sonnet
- **Reasoning**: Testing and quality assurance benefit from good attention to detail
- **Upgrade**: Claude 4 Sonnet for complex testing strategies

#### Reviewer
- **Recommended**: Claude 4 Sonnet
- **Reasoning**: Code and design review benefit from strong analytical capabilities
- **Fallback**: Claude 4 Sonnet for standard reviews

#### Tester
- **Recommended**: Claude 4 Sonnet or Claude 4 Sonnet
- **Reasoning**: Test strategy benefits from solid reasoning capabilities
- **Upgrade**: Claude 4 Sonnet for comprehensive testing approaches

### Coordination Roles

#### Project Manager (PM)
- **Recommended**: Claude 4 Sonnet or Claude 4 Sonnet
- **Reasoning**: Coordination tasks benefit from fast response and good organization
- **Upgrade**: Claude 4 Sonnet for complex project coordination

#### Documenter
- **Recommended**: Claude 4 Sonnet or Claude 4 Sonnet
- **Reasoning**: Documentation benefits from clarity and good writing capabilities
- **Upgrade**: Claude 4 Sonnet for complex technical documentation

#### UX Designer
- **Recommended**: Claude 4 Sonnet
- **Reasoning**: UX design benefits from creative and analytical thinking
- **Fallback**: Claude 4 Sonnet for simple UI tasks

## Selection Strategies

### By Project Complexity

#### Simple Projects
- **Primary Model**: Claude 4 Sonnet
- **Complex Roles**: Claude 4 Sonnet (Architect, Engineer)
- **Research/Analysis**: Claude 4 Sonnet

#### Medium Complexity Projects
- **Primary Model**: Claude 4 Sonnet
- **Architecture/Research**: Claude 4 Sonnet or Claude 4 Opus (for critical decisions)
- **Implementation**: Claude 4 Sonnet
- **Coordination**: Claude 4 Sonnet

#### High Complexity Projects  
- **Architecture/Research**: Claude 4 Opus
- **Implementation**: Claude 4 Sonnet
- **Quality/Review**: Claude 4 Sonnet or Claude 4 Sonnet
- **Coordination**: Claude 4 Sonnet or Claude 4 Sonnet

### By Cost Sensitivity

#### Cost-Optimized
- **Default**: Claude 4 Sonnet
- **Critical Roles Only**: Claude 4 Sonnet (Architect, Engineer)
- **Research**: Claude 4 Sonnet

#### Balanced
- **Default**: Claude 4 Sonnet
- **Research/Analysis**: Claude 4 Opus or Claude 4 Sonnet
- **Coordination**: Claude 4 Sonnet

#### Performance-Optimized
- **Architecture/Research**: Claude 4 Opus
- **Development**: Claude 4 Sonnet or Claude 4 Sonnet
- **Everything Else**: Claude 4 Sonnet

## Configuration Examples

### MCP Tool Configuration

```json
{
  "objective": "Build enterprise application",
  "tasks": [
    {
      "description": "Design solution architecture",
      "role": "solution_architect", 
      "model": "claude-opus-4-20250514"
    },
    {
      "description": "Implement core services",
      "role": "backend_engineer",
      "model": "claude-sonnet-4-20250514"
    },
    {
      "description": "Document API endpoints",
      "role": "documenter",
      "model": "claude-sonnet-4-20250514"
    }
  ]
}
```

### Role Default Overrides

```json
{
  "role_model_defaults": {
    "orchestrator": "claude-opus-4-20250514",
    "solution_architect": "claude-opus-4-20250514",
    "application_architect": "claude-sonnet-4-20250514",
    "data_architect": "claude-opus-4-20250514",
    "api_architect": "claude-sonnet-4-20250514",
    "security_architect": "claude-opus-4-20250514",
    "backend_engineer": "claude-sonnet-4-20250514",
    "frontend_engineer": "claude-sonnet-4-20250514",
    "mobile_engineer": "claude-sonnet-4-20250514",
    "researcher": "claude-opus-4-20250514",
    "data_scientist": "claude-opus-4-20250514",
    "analyst": "claude-sonnet-4-20250514",
    "pm": "claude-sonnet-4-20250514",
    "documenter": "claude-sonnet-4-20250514"
  }
}
```

## Performance Considerations

### Speed vs Quality Trade-offs
- **Claude 4 Sonnet**: Very fast, surprisingly capable for coordination and simple tasks
- **Claude 4 Sonnet**: Proven performance, excellent for most development work
- **Claude 4 Sonnet**: Superior reasoning and coding, excellent for technical work
- **Claude 4 Opus**: Ultimate reasoning capability, use for most complex and critical decisions

### Cost Optimization Tips
1. **Use Claude 4 Sonnet for coordination and simple tasks**
2. **Default to Claude 4 Sonnet for standard development work**
3. **Use Claude 4 Sonnet for complex technical implementation**
4. **Reserve Claude 4 Opus for the most complex analysis and critical architectural decisions**
5. **Consider project budget when selecting default models**

### Context Window Considerations
- All models have sufficient context for typical MAOS tasks
- Large codebases may require careful context management regardless of model
- File-based communication helps manage context across agents

## Model Selection Best Practices

1. **Start Conservative**: Begin with Claude 4 Sonnet and Claude 4 Sonnet, upgrade as complexity increases
2. **Monitor Performance**: Track which roles benefit from Claude 4 Sonnet and Claude 4 Opus capabilities
3. **Cost Awareness**: Balance premium model costs against performance and quality gains
4. **Role Matching**: Use Claude 4 Opus for ultimate reasoning, Claude 4 Sonnet for complex technical work, Claude 4 Sonnet for coordination
5. **Project Adaptation**: Scale model selection with project complexity and criticality

## Future Considerations

- **Model Updates**: New model releases may change recommendations
- **Cost Changes**: Pricing updates may affect optimal selections
- **Performance Improvements**: Newer models may shift the balance between speed/cost/quality
- **Role Evolution**: New agent roles may have different model requirements

---
*Last Updated: 2025-07-13*
*Author: Marvin (Claude)*
*Reviewers: @clafollett (Cali LaFollett - LaFollett Labs LLC)*