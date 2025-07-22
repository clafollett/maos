use crate::aggregates::{Session, SessionStatus, Agent, AgentStatus};
use crate::repositories::{
    AgentRepository, AgentRepositoryError, InstanceRepository, InstanceRepositoryError,
    SessionRepository, SessionRepositoryError,
};
use crate::value_objects::{AgentId, AgentRole, SessionId};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur during context operations
#[derive(Debug, Error)]
pub enum SessionContextError {
    #[error("Repository error: {source}")]
    RepositoryError { source: String },
    
    #[error("Session not found: {session_id}")]
    SessionNotFound { session_id: SessionId },
    
    #[error("Agent not found: {agent_id}")]
    AgentNotFound { agent_id: AgentId },
    
    #[error("Context data serialization error: {message}")]
    SerializationError { message: String },
    
    #[error("Invalid context operation: {message}")]
    InvalidOperation { message: String },
    
    #[error("Context analysis error: {message}")]
    AnalysisError { message: String },
}

impl From<AgentRepositoryError> for SessionContextError {
    fn from(err: AgentRepositoryError) -> Self {
        SessionContextError::RepositoryError {
            source: err.to_string(),
        }
    }
}

impl From<SessionRepositoryError> for SessionContextError {
    fn from(err: SessionRepositoryError) -> Self {
        SessionContextError::RepositoryError {
            source: err.to_string(),
        }
    }
}

impl From<InstanceRepositoryError> for SessionContextError {
    fn from(err: InstanceRepositoryError) -> Self {
        SessionContextError::RepositoryError {
            source: err.to_string(),
        }
    }
}

/// Session context data for decision making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub session_id: SessionId,
    pub task_description: String,
    pub current_status: SessionStatus,
    pub phase_progress: PhaseProgress,
    pub agent_assignments: Vec<AgentAssignment>,
    pub resource_utilization: ResourceUtilization,
    pub timing_metrics: TimingMetrics,
    pub success_indicators: SuccessIndicators,
    pub risk_assessment: RiskAssessment,
    pub recommendations: Vec<Recommendation>,
    pub last_updated: DateTime<Utc>,
}

/// Phase progression information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseProgress {
    pub current_phase: u32,
    pub total_phases: Option<u32>,
    pub phase_start_time: Option<DateTime<Utc>>,
    pub estimated_completion_time: Option<DateTime<Utc>>,
    pub completion_percentage: f32,
    pub phases_completed: u32,
    pub average_phase_duration: Option<Duration>,
}

/// Agent assignment information with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAssignment {
    pub agent_id: AgentId,
    pub role: String,
    pub status: String,
    pub assigned_at: DateTime<Utc>,
    pub capabilities: Vec<String>,
    pub performance_score: f32, // 0.0-1.0
    pub workload_level: WorkloadLevel,
    pub effectiveness_rating: f32, // 0.0-1.0
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    pub active_agents: usize,
    pub max_agents: usize,
    pub agent_utilization_percentage: f32,
    pub instance_count: usize,
    pub memory_usage_estimate: Option<u64>, // MB
    pub cpu_usage_estimate: Option<f32>,    // percentage
}

/// Timing and performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingMetrics {
    pub session_age: Duration,
    pub time_in_current_status: Duration,
    pub estimated_remaining_time: Option<Duration>,
    pub timeout_remaining: Option<Duration>,
    pub average_response_time: Option<Duration>,
    pub idle_time: Duration,
}

/// Success indicators and quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessIndicators {
    pub progress_velocity: f32,    // phases per hour
    pub quality_score: f32,        // 0.0-1.0
    pub agent_satisfaction: f32,   // 0.0-1.0 (mock metric for future)
    pub complexity_handled: f32,   // 0.0-1.0
    pub resource_efficiency: f32,  // 0.0-1.0
}

/// Risk assessment for the session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk_level: RiskLevel,
    pub timeout_risk: f32,           // 0.0-1.0
    pub resource_exhaustion_risk: f32, // 0.0-1.0
    pub agent_failure_risk: f32,     // 0.0-1.0
    pub complexity_risk: f32,        // 0.0-1.0
    pub identified_risks: Vec<IdentifiedRisk>,
}

/// Individual risk identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifiedRisk {
    pub risk_type: String,
    pub severity: RiskLevel,
    pub probability: f32,    // 0.0-1.0
    pub impact: f32,         // 0.0-1.0
    pub description: String,
    pub mitigation: Option<String>,
}

/// Risk severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Workload assessment for agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkloadLevel {
    Light,    // <25% utilization
    Moderate, // 25-50% utilization
    Heavy,    // 50-80% utilization
    Critical, // >80% utilization
}

/// Contextual recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub category: RecommendationCategory,
    pub priority: RecommendationPriority,
    pub title: String,
    pub description: String,
    pub action: String,
    pub expected_benefit: String,
    pub effort_required: EffortLevel,
}

/// Recommendation categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    Performance,
    Resource,
    Risk,
    Quality,
    Strategy,
}

/// Recommendation priorities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Immediate,
    High,
    Medium,
    Low,
}

/// Effort required for recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    Minimal,
    Low,
    Medium,
    High,
}

/// Configuration for session context service
#[derive(Debug, Clone)]
pub struct SessionContextConfig {
    pub update_interval_seconds: u64,
    pub risk_analysis_enabled: bool,
    pub performance_tracking_enabled: bool,
    pub recommendation_engine_enabled: bool,
    pub context_history_retention_hours: u32,
}

impl Default for SessionContextConfig {
    fn default() -> Self {
        Self {
            update_interval_seconds: 30,
            risk_analysis_enabled: true,
            performance_tracking_enabled: true,
            recommendation_engine_enabled: true,
            context_history_retention_hours: 168, // 1 week
        }
    }
}

/// Session context service - provides contextual intelligence for decision making
pub struct SessionContextService {
    agent_repository: Arc<dyn AgentRepository>,
    session_repository: Arc<dyn SessionRepository>,
    instance_repository: Arc<dyn InstanceRepository>,
    config: SessionContextConfig,
    context_cache: std::sync::RwLock<HashMap<SessionId, SessionContext>>,
}

impl SessionContextService {
    pub fn new(
        agent_repository: Arc<dyn AgentRepository>,
        session_repository: Arc<dyn SessionRepository>,
        instance_repository: Arc<dyn InstanceRepository>,
        config: SessionContextConfig,
    ) -> Self {
        Self {
            agent_repository,
            session_repository,
            instance_repository,
            config,
            context_cache: std::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Get comprehensive context for a session
    pub async fn get_session_context(&self, session_id: SessionId) -> Result<SessionContext, SessionContextError> {
        // Check cache first
        if let Some(cached_context) = self.get_cached_context(session_id) {
            let age = Utc::now() - cached_context.last_updated;
            if age.num_seconds() < self.config.update_interval_seconds as i64 {
                return Ok(cached_context);
            }
        }

        // Build fresh context
        let context = self.build_session_context(session_id).await?;
        
        // Update cache
        self.update_context_cache(session_id, context.clone());
        
        Ok(context)
    }

    /// Force refresh context (bypassing cache)
    pub async fn refresh_session_context(&self, session_id: SessionId) -> Result<SessionContext, SessionContextError> {
        let context = self.build_session_context(session_id).await?;
        self.update_context_cache(session_id, context.clone());
        Ok(context)
    }

    /// Get context for all active sessions
    pub async fn get_all_active_contexts(&self) -> Result<Vec<SessionContext>, SessionContextError> {
        let active_sessions = self.session_repository.find_active_sessions().await?;
        let mut contexts = Vec::new();
        
        for session in active_sessions {
            match self.get_session_context(session.id()).await {
                Ok(context) => contexts.push(context),
                Err(e) => {
                    // Log error but continue with other sessions
                    eprintln!("Failed to get context for session {}: {}", session.id(), e);
                }
            }
        }
        
        Ok(contexts)
    }

    /// Analyze session performance trends
    pub async fn analyze_performance_trends(&self, session_id: SessionId) -> Result<Vec<String>, SessionContextError> {
        let context = self.get_session_context(session_id).await?;
        let mut insights = Vec::new();

        // Analyze progress velocity
        if context.success_indicators.progress_velocity > 2.0 {
            insights.push("Session is progressing faster than average".to_string());
        } else if context.success_indicators.progress_velocity < 0.5 {
            insights.push("Session progress is slower than expected".to_string());
        }

        // Analyze resource efficiency
        if context.success_indicators.resource_efficiency > 0.8 {
            insights.push("Resource utilization is highly efficient".to_string());
        } else if context.success_indicators.resource_efficiency < 0.4 {
            insights.push("Resource utilization could be improved".to_string());
        }

        // Analyze agent performance
        let avg_performance = context.agent_assignments.iter()
            .map(|a| a.performance_score)
            .sum::<f32>() / context.agent_assignments.len() as f32;
            
        if avg_performance > 0.8 {
            insights.push("Agent performance is excellent".to_string());
        } else if avg_performance < 0.5 {
            insights.push("Agent performance needs attention".to_string());
        }

        Ok(insights)
    }

    /// Get recommendations for session optimization
    pub async fn get_optimization_recommendations(&self, session_id: SessionId) -> Result<Vec<Recommendation>, SessionContextError> {
        if !self.config.recommendation_engine_enabled {
            return Ok(Vec::new());
        }

        let context = self.get_session_context(session_id).await?;
        let mut recommendations = Vec::new();

        // Resource optimization recommendations
        if context.resource_utilization.agent_utilization_percentage < 50.0 {
            recommendations.push(Recommendation {
                category: RecommendationCategory::Resource,
                priority: RecommendationPriority::Medium,
                title: "Underutilized Agent Capacity".to_string(),
                description: "Current agent utilization is below 50%".to_string(),
                action: "Consider assigning additional tasks or reducing agent count".to_string(),
                expected_benefit: "Improved resource efficiency and cost reduction".to_string(),
                effort_required: EffortLevel::Low,
            });
        }

        // Performance recommendations
        if context.success_indicators.progress_velocity < 1.0 {
            recommendations.push(Recommendation {
                category: RecommendationCategory::Performance,
                priority: RecommendationPriority::High,
                title: "Slow Progress Velocity".to_string(),
                description: "Session is progressing slower than expected".to_string(),
                action: "Review agent assignments and task complexity".to_string(),
                expected_benefit: "Faster session completion".to_string(),
                effort_required: EffortLevel::Medium,
            });
        }

        // Risk mitigation recommendations
        if context.risk_assessment.overall_risk_level == RiskLevel::High {
            recommendations.push(Recommendation {
                category: RecommendationCategory::Risk,
                priority: RecommendationPriority::Immediate,
                title: "High Risk Level Detected".to_string(),
                description: "Session has high overall risk".to_string(),
                action: "Review risk factors and implement mitigation strategies".to_string(),
                expected_benefit: "Reduced risk of session failure".to_string(),
                effort_required: EffortLevel::High,
            });
        }

        // Timeout recommendations
        if let Some(timeout_remaining) = context.timing_metrics.timeout_remaining {
            if timeout_remaining.num_minutes() < 10 {
                recommendations.push(Recommendation {
                    category: RecommendationCategory::Strategy,
                    priority: RecommendationPriority::Immediate,
                    title: "Session Timeout Approaching".to_string(),
                    description: format!("Only {} minutes remaining", timeout_remaining.num_minutes()),
                    action: "Complete current tasks or extend session timeout".to_string(),
                    expected_benefit: "Avoid session timeout failure".to_string(),
                    effort_required: EffortLevel::Minimal,
                });
            }
        }

        Ok(recommendations)
    }

    /// Compare sessions for benchmarking
    pub async fn compare_sessions(&self, session_ids: Vec<SessionId>) -> Result<HashMap<String, Vec<f32>>, SessionContextError> {
        let mut metrics = HashMap::new();
        let mut progress_velocities = Vec::new();
        let mut quality_scores = Vec::new();
        let mut resource_efficiencies = Vec::new();
        let mut agent_utilizations = Vec::new();

        for session_id in session_ids {
            let context = self.get_session_context(session_id).await?;
            progress_velocities.push(context.success_indicators.progress_velocity);
            quality_scores.push(context.success_indicators.quality_score);
            resource_efficiencies.push(context.success_indicators.resource_efficiency);
            agent_utilizations.push(context.resource_utilization.agent_utilization_percentage);
        }

        metrics.insert("progress_velocity".to_string(), progress_velocities);
        metrics.insert("quality_score".to_string(), quality_scores);
        metrics.insert("resource_efficiency".to_string(), resource_efficiencies);
        metrics.insert("agent_utilization".to_string(), agent_utilizations);

        Ok(metrics)
    }

    // Private helper methods

    async fn build_session_context(&self, session_id: SessionId) -> Result<SessionContext, SessionContextError> {
        let session = self.session_repository
            .find_by_id(session_id)
            .await?
            .ok_or(SessionContextError::SessionNotFound { session_id })?;

        let now = Utc::now();

        // Build phase progress
        let phase_progress = self.build_phase_progress(&session).await?;

        // Build agent assignments
        let agent_assignments = self.build_agent_assignments(&session).await?;

        // Build resource utilization
        let resource_utilization = self.build_resource_utilization(&session).await?;

        // Build timing metrics
        let timing_metrics = self.build_timing_metrics(&session, now).await?;

        // Build success indicators
        let success_indicators = self.build_success_indicators(&session, &agent_assignments, &timing_metrics).await?;

        // Build risk assessment
        let risk_assessment = self.build_risk_assessment(&session, &timing_metrics, &resource_utilization).await?;

        // Generate recommendations
        let recommendations = if self.config.recommendation_engine_enabled {
            self.generate_recommendations(&session, &success_indicators, &risk_assessment, &resource_utilization).await?
        } else {
            Vec::new()
        };

        Ok(SessionContext {
            session_id,
            task_description: session.task_description().to_string(),
            current_status: session.status().clone(),
            phase_progress,
            agent_assignments,
            resource_utilization,
            timing_metrics,
            success_indicators,
            risk_assessment,
            recommendations,
            last_updated: now,
        })
    }

    async fn build_phase_progress(&self, session: &Session) -> Result<PhaseProgress, SessionContextError> {
        let current_phase = session.phase_count();
        let total_phases = session.total_phases();
        
        let completion_percentage = if let Some(total) = total_phases {
            if total > 0 {
                (current_phase as f32 / total as f32) * 100.0
            } else {
                0.0
            }
        } else {
            // Estimate based on time if no total phases set
            if let Some(started_at) = session.started_at() {
                let elapsed = Utc::now() - started_at;
                let estimated_total_time = Duration::hours(2); // Default estimate
                (elapsed.num_seconds() as f32 / estimated_total_time.num_seconds() as f32) * 100.0
            } else {
                0.0
            }
        };

        Ok(PhaseProgress {
            current_phase,
            total_phases,
            phase_start_time: session.started_at(),
            estimated_completion_time: None, // TODO: Implement estimation logic
            completion_percentage: completion_percentage.min(100.0),
            phases_completed: current_phase,
            average_phase_duration: None, // TODO: Calculate from history
        })
    }

    async fn build_agent_assignments(&self, session: &Session) -> Result<Vec<AgentAssignment>, SessionContextError> {
        let mut assignments = Vec::new();
        
        for (agent_id, agent_info) in session.active_agents() {
            if let Some(agent) = self.agent_repository.find_by_id(*agent_id).await? {
                // Get instance count for workload assessment
                let active_instances = self.instance_repository.find_active_by_agent(*agent_id).await?;
                let workload_level = match active_instances.len() {
                    0 => WorkloadLevel::Light,
                    1 => WorkloadLevel::Moderate,
                    2..=3 => WorkloadLevel::Heavy,
                    _ => WorkloadLevel::Critical,
                };

                assignments.push(AgentAssignment {
                    agent_id: *agent_id,
                    role: agent_info.role_name.clone(),
                    status: format!("{:?}", agent_info.status),
                    assigned_at: agent_info.spawned_at,
                    capabilities: agent.capabilities.clone(),
                    performance_score: 0.8, // Mock score - would be calculated from metrics
                    workload_level,
                    effectiveness_rating: 0.75, // Mock rating - would be calculated from outcomes
                });
            }
        }

        Ok(assignments)
    }

    async fn build_resource_utilization(&self, session: &Session) -> Result<ResourceUtilization, SessionContextError> {
        let active_agents = session.active_agents().len();
        let max_agents = session.metadata().max_agents;
        let utilization_percentage = if max_agents > 0 {
            (active_agents as f32 / max_agents as f32) * 100.0
        } else {
            0.0
        };

        // Get total instance count across all agents in session
        let mut total_instances = 0;
        for agent_id in session.active_agents().keys() {
            let instances = self.instance_repository.find_active_by_agent(*agent_id).await?;
            total_instances += instances.len();
        }

        Ok(ResourceUtilization {
            active_agents,
            max_agents,
            agent_utilization_percentage: utilization_percentage,
            instance_count: total_instances,
            memory_usage_estimate: Some(total_instances as u64 * 512), // Mock: 512MB per instance
            cpu_usage_estimate: Some(utilization_percentage * 0.6), // Mock: 60% correlation
        })
    }

    async fn build_timing_metrics(&self, session: &Session, now: DateTime<Utc>) -> Result<TimingMetrics, SessionContextError> {
        let session_age = now - session.created_at();
        
        let time_in_current_status = now - session.updated_at();
        
        let timeout_remaining = if let Some(started_at) = session.started_at() {
            let timeout_duration = Duration::minutes(session.metadata().timeout_minutes as i64);
            let elapsed = now - started_at;
            if elapsed < timeout_duration {
                Some(timeout_duration - elapsed)
            } else {
                Some(Duration::zero())
            }
        } else {
            None
        };

        Ok(TimingMetrics {
            session_age,
            time_in_current_status,
            estimated_remaining_time: None, // TODO: Implement estimation
            timeout_remaining,
            average_response_time: None, // TODO: Calculate from metrics
            idle_time: Duration::minutes(2), // Mock value
        })
    }

    async fn build_success_indicators(&self, session: &Session, assignments: &[AgentAssignment], timing: &TimingMetrics) -> Result<SuccessIndicators, SessionContextError> {
        let progress_velocity = if session.phase_count() > 0 && timing.session_age.num_hours() > 0 {
            session.phase_count() as f32 / timing.session_age.num_hours() as f32
        } else {
            0.0
        };

        let avg_performance = if !assignments.is_empty() {
            assignments.iter().map(|a| a.performance_score).sum::<f32>() / assignments.len() as f32
        } else {
            0.0
        };

        let resource_efficiency = match session.status() {
            SessionStatus::InProgress => 0.7, // Mock calculation
            SessionStatus::Completed => 0.85,
            _ => 0.5,
        };

        Ok(SuccessIndicators {
            progress_velocity,
            quality_score: avg_performance,
            agent_satisfaction: 0.8, // Mock value
            complexity_handled: 0.6, // Mock value based on task description analysis
            resource_efficiency,
        })
    }

    async fn build_risk_assessment(&self, session: &Session, timing: &TimingMetrics, resource: &ResourceUtilization) -> Result<RiskAssessment, SessionContextError> {
        let mut risks = Vec::new();

        // Timeout risk
        let timeout_risk = if let Some(remaining) = timing.timeout_remaining {
            let total_timeout = Duration::minutes(session.metadata().timeout_minutes as i64);
            let elapsed_ratio = 1.0 - (remaining.num_seconds() as f32 / total_timeout.num_seconds() as f32);
            elapsed_ratio.max(0.0).min(1.0)
        } else {
            0.0
        };

        if timeout_risk > 0.8 {
            risks.push(IdentifiedRisk {
                risk_type: "Timeout".to_string(),
                severity: RiskLevel::High,
                probability: timeout_risk,
                impact: 0.9,
                description: "Session is approaching timeout".to_string(),
                mitigation: Some("Extend timeout or complete current tasks".to_string()),
            });
        }

        // Resource exhaustion risk
        let resource_risk = resource.agent_utilization_percentage / 100.0;
        if resource_risk > 0.9 {
            risks.push(IdentifiedRisk {
                risk_type: "Resource Exhaustion".to_string(),
                severity: RiskLevel::Medium,
                probability: resource_risk - 0.5,
                impact: 0.7,
                description: "Agent utilization is very high".to_string(),
                mitigation: Some("Add more agents or reduce workload".to_string()),
            });
        }

        // Agent failure risk (based on workload)
        let agent_failure_risk = if resource.active_agents > 0 {
            resource.instance_count as f32 / (resource.active_agents * 3) as f32 // 3 instances per agent is max
        } else {
            0.0
        };

        let overall_risk_level = if timeout_risk > 0.8 || agent_failure_risk > 0.8 {
            RiskLevel::High
        } else if timeout_risk > 0.6 || resource_risk > 0.8 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        Ok(RiskAssessment {
            overall_risk_level,
            timeout_risk,
            resource_exhaustion_risk: resource_risk,
            agent_failure_risk,
            complexity_risk: 0.4, // Mock value
            identified_risks: risks,
        })
    }

    async fn generate_recommendations(&self, session: &Session, success: &SuccessIndicators, risk: &RiskAssessment, resource: &ResourceUtilization) -> Result<Vec<Recommendation>, SessionContextError> {
        let mut recommendations = Vec::new();

        // Performance recommendations
        if success.progress_velocity < 0.5 {
            recommendations.push(Recommendation {
                category: RecommendationCategory::Performance,
                priority: RecommendationPriority::High,
                title: "Improve Progress Velocity".to_string(),
                description: "Session is progressing slowly".to_string(),
                action: "Review task complexity and agent assignments".to_string(),
                expected_benefit: "Faster completion and better resource utilization".to_string(),
                effort_required: EffortLevel::Medium,
            });
        }

        // Resource recommendations
        if resource.agent_utilization_percentage > 90.0 {
            recommendations.push(Recommendation {
                category: RecommendationCategory::Resource,
                priority: RecommendationPriority::Medium,
                title: "High Resource Utilization".to_string(),
                description: "Agent resources are near capacity".to_string(),
                action: "Consider adding more agents or optimizing tasks".to_string(),
                expected_benefit: "Reduced bottlenecks and improved reliability".to_string(),
                effort_required: EffortLevel::Low,
            });
        }

        Ok(recommendations)
    }

    fn get_cached_context(&self, session_id: SessionId) -> Option<SessionContext> {
        let cache = self.context_cache.read().ok()?;
        cache.get(&session_id).cloned()
    }

    fn update_context_cache(&self, session_id: SessionId, context: SessionContext) {
        if let Ok(mut cache) = self.context_cache.write() {
            cache.insert(session_id, context);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_context_config_default() {
        let config = SessionContextConfig::default();
        
        assert_eq!(config.update_interval_seconds, 30);
        assert!(config.risk_analysis_enabled);
        assert!(config.performance_tracking_enabled);
        assert!(config.recommendation_engine_enabled);
        assert_eq!(config.context_history_retention_hours, 168);
    }

    #[test]
    fn test_workload_level_classification() {
        // Test workload level enum
        assert_eq!(std::mem::discriminant(&WorkloadLevel::Light), std::mem::discriminant(&WorkloadLevel::Light));
        assert_ne!(std::mem::discriminant(&WorkloadLevel::Light), std::mem::discriminant(&WorkloadLevel::Heavy));
    }

    #[test]
    fn test_risk_level_comparison() {
        assert_eq!(RiskLevel::High, RiskLevel::High);
        assert_ne!(RiskLevel::High, RiskLevel::Low);
    }

    #[test]
    fn test_recommendation_structure() {
        let recommendation = Recommendation {
            category: RecommendationCategory::Performance,
            priority: RecommendationPriority::High,
            title: "Test Recommendation".to_string(),
            description: "This is a test".to_string(),
            action: "Do something".to_string(),
            expected_benefit: "Things will improve".to_string(),
            effort_required: EffortLevel::Low,
        };

        assert_eq!(recommendation.title, "Test Recommendation");
        assert!(matches!(recommendation.priority, RecommendationPriority::High));
    }

    #[test]
    fn test_identified_risk_structure() {
        let risk = IdentifiedRisk {
            risk_type: "Test Risk".to_string(),
            severity: RiskLevel::Medium,
            probability: 0.7,
            impact: 0.8,
            description: "This is a test risk".to_string(),
            mitigation: Some("Mitigate it".to_string()),
        };

        assert_eq!(risk.probability, 0.7);
        assert_eq!(risk.impact, 0.8);
        assert!(risk.mitigation.is_some());
    }
}