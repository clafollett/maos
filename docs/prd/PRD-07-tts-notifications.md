# PRD 7: MAOS TTS & Notifications System

## Executive Summary

The MAOS TTS & Notifications System provides real-time audio feedback and notifications during development sessions through a multi-provider TTS architecture with intelligent fallback chains. This system translates the existing sophisticated Python TTS implementation into a high-performance Rust crate while preserving all functionality and adding enhanced reliability, async execution, and seamless integration with MAOS core types.

**Key Deliverable**: A production-ready `maos-tts` crate that delivers sub-second audio notifications with 99.9% reliability across all supported platforms through a provider abstraction layer with intelligent fallbacks.

## Problem Statement

The current Python-based TTS system, while functional, has several limitations that impact the MAOS user experience:

- **Performance Overhead**: Python subprocess calls and blocking execution impact overall system responsiveness
- **Resource Management**: No proper timeout handling or concurrent TTS request management  
- **Error Handling**: Limited error recovery and provider failover capabilities
- **Integration Complexity**: Loose coupling with MAOS core types and configuration system
- **Platform Dependencies**: Complex dependency management across different TTS providers
- **Notification Reliability**: No guaranteed delivery or retry mechanisms for critical notifications

We need a **native Rust TTS system** that maintains all existing functionality while providing <1s notification delivery, robust error handling, and seamless integration with the MAOS ecosystem.

## Goals & Success Metrics

### Primary Goals

1. **Performance Excellence**: Sub-second notification delivery with non-blocking execution
2. **Provider Reliability**: 99.9% successful TTS delivery through intelligent fallback chains
3. **Developer Experience**: Zero-configuration setup with sensible defaults and personalization
4. **System Integration**: Seamless integration with MAOS core types and configuration management
5. **Cross-Platform Support**: Consistent behavior across Linux, macOS, and Windows

### Success Metrics

- **Notification Speed**: Audio feedback delivered in <1s from trigger event
- **Fallback Success**: Provider fallback completes in <100ms with no user-visible delay
- **Memory Efficiency**: <5MB memory overhead for TTS subsystem
- **Error Recovery**: 99.9% successful delivery rate even with provider failures
- **Configuration Flexibility**: Support for all existing Python configuration options
- **API Stability**: Zero breaking changes to existing notification interfaces

## User Personas & Use Cases

### Primary User: MAOS Developer
**Profile**: Uses MAOS for development orchestration and needs immediate feedback
**Use Case**: Real-time notifications about task completion, errors, and system state changes
**Success Criteria**: Instant, clear audio feedback without interrupting development flow

### Secondary User: MAOS Agent
**Profile**: Automated agent needing to communicate status and request user input
**Use Case**: Announce completion, request user input, alert about errors or security issues
**Success Criteria**: Reliable delivery of critical notifications with engineer name personalization

### Tertiary User: MAOS Administrator
**Profile**: Configures and maintains MAOS installations across different environments
**Use Case**: Flexible TTS provider configuration, voice selection, and troubleshooting
**Success Criteria**: Simple configuration with detailed logging and error reporting

## Functional Requirements

### 1. TTS Provider Abstraction

#### 1.1 Provider Trait Design
```rust
/// Universal trait for TTS providers with async support
#[async_trait]
pub trait TtsProvider: Send + Sync {
    /// Unique identifier for this provider
    fn name(&self) -> &'static str;
    
    /// Provider priority (higher = preferred)
    fn priority(&self) -> u8;
    
    /// Check if provider is available and configured
    async fn is_available(&self) -> bool;
    
    /// Synthesize text to speech
    async fn speak(&self, request: &TtsRequest) -> TtsResult<()>;
    
    /// Get supported voices for this provider
    fn supported_voices(&self) -> Vec<VoiceInfo>;
    
    /// Provider-specific health check
    async fn health_check(&self) -> TtsResult<ProviderHealth>;
}

/// TTS request with full context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsRequest {
    pub text: String,
    pub voice: Option<String>,
    pub session_id: Option<SessionId>,
    pub notification_type: NotificationType,
    pub urgency: NotificationUrgency,
    pub engineer_name: Option<String>,
    pub request_id: String,
}

/// Provider health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub available: bool,
    pub response_time_ms: Option<u64>,
    pub error_message: Option<String>,
    pub last_success: Option<DateTime<Utc>>,
}
```

#### 1.2 Provider Implementations

**ElevenLabs Provider**
```rust
pub struct ElevenLabsProvider {
    client: ElevenLabsClient,
    config: ElevenLabsConfig,
    rate_limiter: RateLimiter,
}

impl ElevenLabsProvider {
    /// Create new ElevenLabs provider with API key
    pub fn new(api_key: String, config: ElevenLabsConfig) -> TtsResult<Self> {
        // Validate API key and initialize client
        // Set up rate limiting (25 requests/minute)
        // Configure voice settings and model selection
    }
    
    /// Get available voices from ElevenLabs API
    pub async fn fetch_voices(&self) -> TtsResult<Vec<VoiceInfo>> {
        // Cache voices for 24 hours to reduce API calls
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevenLabsConfig {
    pub voice_id: String,           // Default: "IKne3meq5aSn9XLyUdCD"
    pub model: String,              // Default: "eleven_turbo_v2_5"
    pub output_format: String,      // Default: "mp3_44100_128"
    pub stability: f32,             // Voice stability (0.0-1.0)
    pub similarity_boost: f32,      // Voice similarity (0.0-1.0)
    pub style: Option<f32>,         // Voice style (0.0-1.0)
    pub use_speaker_boost: bool,    // Enable speaker boost
}
```

**OpenAI Provider**
```rust
pub struct OpenAiProvider {
    client: OpenAiClient,
    config: OpenAiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    pub model: String,              // Default: "tts-1"
    pub voice: String,              // Default: "nova"
    pub response_format: String,    // Default: "mp3"
    pub speed: f32,                 // Speech speed (0.25-4.0)
    pub instructions: Option<String>, // Voice instructions
}
```

**macOS Provider**
```rust
pub struct MacOsProvider {
    config: MacOsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacOsConfig {
    pub voice: String,              // Default: "Alex"
    pub rate: u32,                  // Speech rate (default: 190)
    pub quality: u32,               // Audio quality (0-127)
    pub volume: f32,                // Volume level (0.0-1.0)
}
```

**Pyttsx3 Provider**
```rust
pub struct Pyttsx3Provider {
    config: Pyttsx3Config,
}

#[derive(Debug, Clone, Serialize, Deserialize)]  
pub struct Pyttsx3Config {
    pub voice: String,              // Default: "default"
    pub rate: u32,                  // Speech rate (default: 190)
    pub volume: f32,                // Volume level (0.0-1.0)
    pub engine: Option<String>,     // TTS engine override
}
```

### 2. TTS Manager and Orchestration

#### 2.1 Manager Design
```rust
/// Main TTS orchestration system
pub struct TtsManager {
    providers: Vec<Box<dyn TtsProvider>>,
    config: TtsConfig,
    request_queue: mpsc::Sender<TtsTask>,
    metrics: Arc<TtsMetrics>,
}

impl TtsManager {
    /// Initialize TTS system with all available providers
    pub async fn new(config: TtsConfig) -> TtsResult<Self> {
        let mut providers = Vec::new();
        
        // Initialize providers in priority order
        if let Some(elevenlabs_config) = &config.elevenlabs {
            if let Ok(provider) = ElevenLabsProvider::new(elevenlabs_config.clone()).await {
                providers.push(Box::new(provider) as Box<dyn TtsProvider>);
            }
        }
        
        if let Some(openai_config) = &config.openai {
            if let Ok(provider) = OpenAiProvider::new(openai_config.clone()).await {
                providers.push(Box::new(provider));
            }
        }
        
        // Always add fallback providers
        providers.push(Box::new(MacOsProvider::new(config.macos.clone())));
        providers.push(Box::new(Pyttsx3Provider::new(config.pyttsx3.clone())));
        
        // Sort by priority
        providers.sort_by_key(|p| std::cmp::Reverse(p.priority()));
        
        Ok(Self {
            providers,
            config,
            request_queue: Self::start_processing_loop(),
            metrics: Arc::new(TtsMetrics::new()),
        })
    }
    
    /// Submit TTS request (non-blocking)
    pub async fn speak(&self, request: TtsRequest) -> TtsResult<()> {
        let task = TtsTask {
            request,
            retry_count: 0,
            created_at: Utc::now(),
        };
        
        self.request_queue.send(task).await
            .map_err(|_| TtsError::QueueFull)?;
        
        Ok(())
    }
    
    /// Process TTS requests with fallback logic
    async fn process_request(&self, task: TtsTask) -> TtsResult<()> {
        for provider in &self.providers {
            if !provider.is_available().await {
                continue;
            }
            
            let start_time = Instant::now();
            match provider.speak(&task.request).await {
                Ok(()) => {
                    let duration = start_time.elapsed();
                    self.metrics.record_success(provider.name(), duration).await;
                    return Ok(());
                }
                Err(e) => {
                    self.metrics.record_failure(provider.name(), &e).await;
                    tracing::warn!(
                        provider = provider.name(),
                        error = %e,
                        "TTS provider failed, trying next"
                    );
                    continue;
                }
            }
        }
        
        Err(TtsError::AllProvidersFailed)
    }
}

/// TTS processing task
#[derive(Debug)]
struct TtsTask {
    request: TtsRequest,
    retry_count: u32,
    created_at: DateTime<Utc>,
}
```

#### 2.2 Configuration Integration
```rust
/// TTS configuration integrated with MAOS core config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsConfig {
    pub enabled: bool,
    pub default_provider: Option<String>,
    pub timeout_ms: u64,
    pub max_queue_size: usize,
    pub text_length_limit: usize,
    pub concurrent_requests: usize,
    
    // Provider-specific configurations
    pub elevenlabs: Option<ElevenLabsConfig>,
    pub openai: Option<OpenAiConfig>,
    pub macos: MacOsConfig,
    pub pyttsx3: Pyttsx3Config,
    
    // Notification settings
    pub responses: ResponseTtsConfig,
    pub completion: CompletionTtsConfig,
    pub notifications: NotificationTtsConfig,
    
    // Text processing
    pub text_processing: TextProcessingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTtsConfig {
    pub enabled: bool,
    pub max_length: usize,        // Truncate long responses
    pub include_code: bool,       // Read code blocks aloud
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionTtsConfig {
    pub enabled: bool,
    pub include_agent_type: bool,
    pub include_engineer_name: bool,
    pub custom_template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTtsConfig {
    pub enabled: bool,
    pub urgency_filter: NotificationUrgency,
    pub include_session_context: bool,
}
```

### 3. Text Processing and Optimization

#### 3.1 Text Preprocessing
```rust
/// Text processing for optimal TTS delivery
pub struct TextProcessor {
    config: TextProcessingConfig,
    abbreviation_map: HashMap<String, String>,
}

impl TextProcessor {
    /// Clean and optimize text for speech synthesis
    pub fn process_for_speech(&self, text: &str) -> String {
        let mut processed = text.to_string();
        
        // Remove or replace problematic characters
        processed = self.clean_technical_content(&processed);
        processed = self.expand_abbreviations(&processed);
        processed = self.format_numbers(&processed);
        processed = self.optimize_punctuation(&processed);
        processed = self.truncate_if_needed(&processed);
        
        processed
    }
    
    /// Remove code blocks and technical syntax
    fn clean_technical_content(&self, text: &str) -> String {
        // Remove markdown code blocks
        let code_block_regex = regex::Regex::new(r"```[\s\S]*?```").unwrap();
        let mut cleaned = code_block_regex.replace_all(text, "").to_string();
        
        // Remove inline code
        let inline_code_regex = regex::Regex::new(r"`[^`]+`").unwrap();
        cleaned = inline_code_regex.replace_all(&cleaned, "").to_string();
        
        // Clean file paths and URLs
        let path_regex = regex::Regex::new(r"/[^\s]+|https?://[^\s]+").unwrap();
        cleaned = path_regex.replace_all(&cleaned, "file path").to_string();
        
        cleaned
    }
    
    /// Expand common abbreviations for clarity
    fn expand_abbreviations(&self, text: &str) -> String {
        let mut expanded = text.to_string();
        
        for (abbrev, expansion) in &self.abbreviation_map {
            expanded = expanded.replace(abbrev, expansion);
        }
        
        expanded
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextProcessingConfig {
    pub max_length: usize,
    pub remove_code_blocks: bool,
    pub expand_abbreviations: bool,
    pub format_numbers: bool,
    pub optimize_punctuation: bool,
}
```

### 4. Notification System Integration

#### 4.1 Notification Commands
```rust
/// High-level notification interface
pub struct NotificationSystem {
    tts_manager: Arc<TtsManager>,
    config: MaosConfig,
}

impl NotificationSystem {
    /// Send notification with automatic TTS
    pub async fn notify(&self, message: NotificationMessage) -> MaosResult<()> {
        // Log notification
        self.log_notification(&message).await?;
        
        // Generate TTS request if enabled
        if self.should_speak(&message) {
            let tts_request = self.build_tts_request(&message)?;
            self.tts_manager.speak(tts_request).await?;
        }
        
        Ok(())
    }
    
    /// Announce task completion
    pub async fn announce_completion(
        &self,
        agent_type: &AgentType,
        session_id: &SessionId,
        task_description: Option<&str>,
    ) -> MaosResult<()> {
        let message = self.build_completion_message(agent_type, session_id, task_description);
        self.notify(message).await
    }
    
    /// Request user input with announcement
    pub async fn request_user_input(
        &self,
        session_id: &SessionId,
        agent_id: &AgentId,
        prompt: &str,
    ) -> MaosResult<()> {
        let message = self.build_input_request_message(session_id, agent_id, prompt);
        self.notify(message).await
    }
    
    /// Emergency stop announcement
    pub async fn announce_stop(
        &self,
        session_id: &SessionId,
        reason: &str,
    ) -> MaosResult<()> {
        let message = NotificationMessage {
            message: format!("MAOS session stopped: {}", reason),
            notification_type: NotificationType::SystemError,
            urgency: NotificationUrgency::High,
            session_id: Some(session_id.clone()),
            engineer_name: self.get_engineer_name(),
        };
        
        self.notify(message).await
    }
}
```

#### 4.2 CLI Integration
```rust
/// CLI commands for TTS and notifications
#[derive(Debug, Clone, Parser)]
pub enum TtsCommand {
    /// Send a notification with TTS
    #[command(name = "notify")]
    Notify {
        /// Message to announce
        message: String,
        
        /// Notification urgency level
        #[arg(long, default_value = "normal")]
        urgency: NotificationUrgency,
        
        /// Include engineer name
        #[arg(long)]
        personal: bool,
    },
    
    /// Test TTS system
    #[command(name = "test")]
    Test {
        /// Test message
        #[arg(default_value = "MAOS TTS system test")]
        message: String,
        
        /// Specific provider to test
        #[arg(long)]
        provider: Option<String>,
    },
    
    /// Stop all TTS playback
    #[command(name = "stop")]
    Stop,
    
    /// Show TTS system status
    #[command(name = "status")]
    Status,
    
    /// List available voices
    #[command(name = "voices")]
    Voices {
        /// Provider to list voices for
        #[arg(long)]
        provider: Option<String>,
    },
}
```

### 5. Error Handling and Recovery

#### 5.1 Error Types
```rust
/// TTS-specific error types
#[derive(thiserror::Error, Debug)]
pub enum TtsError {
    #[error("TTS provider not available: {provider}")]
    ProviderUnavailable { provider: String },
    
    #[error("All TTS providers failed")]
    AllProvidersFailed,
    
    #[error("TTS request queue is full")]
    QueueFull,
    
    #[error("TTS timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
    
    #[error("Invalid voice configuration: {details}")]
    InvalidVoice { details: String },
    
    #[error("Audio playback failed: {source}")]
    PlaybackFailed { source: Box<dyn std::error::Error + Send + Sync> },
    
    #[error("TTS API error: {message}")]
    ApiError { message: String, status_code: Option<u16> },
    
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Text processing error: {0}")]
    TextProcessing(String),
}

pub type TtsResult<T> = Result<T, TtsError>;
```

#### 5.2 Recovery Strategies
```rust
/// TTS system recovery and health monitoring
pub struct TtsHealthMonitor {
    manager: Arc<TtsManager>,
    health_check_interval: Duration,
    recovery_strategies: HashMap<String, RecoveryStrategy>,
}

impl TtsHealthMonitor {
    /// Start health monitoring background task
    pub async fn start_monitoring(&self) {
        let mut interval = tokio::time::interval(self.health_check_interval);
        
        loop {
            interval.tick().await;
            self.perform_health_checks().await;
        }
    }
    
    /// Check health of all providers and trigger recovery if needed
    async fn perform_health_checks(&self) {
        for provider in &self.manager.providers {
            match provider.health_check().await {
                Ok(health) if !health.available => {
                    self.trigger_recovery(provider.name()).await;
                }
                Err(e) => {
                    tracing::error!(
                        provider = provider.name(),
                        error = %e,
                        "Provider health check failed"
                    );
                }
                _ => {} // Provider is healthy
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    Restart,
    Reinitialize,
    Skip,
    Fallback { to_provider: String },
}
```

## Non-Functional Requirements

### Performance Requirements
- **Notification Latency**: TTS requests processed and queued in <10ms
- **Audio Delivery**: First audio output within 1s of request
- **Fallback Speed**: Provider fallback completes in <100ms
- **Memory Usage**: <5MB steady-state memory consumption
- **CPU Usage**: <5% CPU during TTS generation
- **Concurrent Requests**: Support 10 concurrent TTS requests

### Reliability Requirements
- **Uptime**: 99.9% successful TTS delivery rate
- **Error Recovery**: Automatic recovery from provider failures
- **Graceful Degradation**: System continues with reduced functionality if all TTS providers fail
- **Request Durability**: Queued requests survive brief system interruptions

### Security Requirements
- **API Key Protection**: Secure storage and transmission of provider API keys
- **Input Validation**: All text input sanitized to prevent injection attacks
- **Rate Limiting**: Proper rate limiting to prevent provider quota exhaustion
- **Audio Security**: No persistent storage of synthesized audio

### Compatibility Requirements
- **Platform Support**: Linux, macOS, Windows (all architectures)
- **Rust Version**: Compatible with Rust 2024 edition
- **Provider APIs**: Support latest versions of ElevenLabs and OpenAI TTS APIs
- **Configuration**: Backward compatible with existing Python configuration

## Technical Design

### 1. Crate Architecture
```
maos-tts/
├── src/
│   ├── lib.rs                  # Public API exports
│   ├── manager.rs              # TTS orchestration manager
│   ├── providers/              # TTS provider implementations
│   │   ├── mod.rs
│   │   ├── elevenlabs.rs       # ElevenLabs TTS provider
│   │   ├── openai.rs           # OpenAI TTS provider
│   │   ├── macos.rs            # macOS native TTS
│   │   ├── pyttsx3.rs          # pyttsx3 fallback
│   │   └── traits.rs           # Provider trait definitions
│   ├── processor/              # Text processing
│   │   ├── mod.rs
│   │   ├── cleaner.rs          # Text cleaning and formatting
│   │   └── config.rs           # Processing configuration
│   ├── notification/           # Notification system
│   │   ├── mod.rs
│   │   ├── system.rs           # High-level notification interface
│   │   └── templates.rs        # Message templates
│   ├── config/                 # Configuration management
│   │   ├── mod.rs
│   │   ├── types.rs            # Configuration types
│   │   └── validation.rs       # Config validation
│   ├── error.rs                # Error types and handling
│   ├── metrics.rs              # Performance monitoring
│   └── health.rs               # Health monitoring and recovery
├── tests/
│   ├── integration/            # Integration tests
│   ├── providers/              # Provider-specific tests
│   └── common/                 # Test utilities
└── benches/                    # Performance benchmarks
```

### 2. Async Architecture Design

#### 2.1 Request Processing Pipeline
```rust
/// Async TTS processing pipeline
pub struct TtsProcessingPipeline {
    request_receiver: mpsc::Receiver<TtsTask>,
    provider_pool: Arc<ProviderPool>,
    text_processor: Arc<TextProcessor>,
    metrics: Arc<TtsMetrics>,
}

impl TtsProcessingPipeline {
    /// Main processing loop
    pub async fn process_requests(&mut self) {
        let mut concurrent_tasks = JoinSet::new();
        
        while let Some(task) = self.request_receiver.recv().await {
            // Limit concurrent TTS requests
            if concurrent_tasks.len() >= self.max_concurrent_requests {
                // Wait for a task to complete
                concurrent_tasks.join_next().await;
            }
            
            // Spawn task for async processing
            let provider_pool = Arc::clone(&self.provider_pool);
            let text_processor = Arc::clone(&self.text_processor);
            let metrics = Arc::clone(&self.metrics);
            
            concurrent_tasks.spawn(async move {
                Self::process_single_task(task, provider_pool, text_processor, metrics).await
            });
        }
    }
    
    /// Process single TTS task with fallback
    async fn process_single_task(
        mut task: TtsTask,
        provider_pool: Arc<ProviderPool>,
        text_processor: Arc<TextProcessor>,
        metrics: Arc<TtsMetrics>,
    ) -> TtsResult<()> {
        // Process text for optimal speech synthesis
        task.request.text = text_processor.process_for_speech(&task.request.text);
        
        // Try providers in priority order
        for provider in provider_pool.get_available_providers().await {
            let start_time = Instant::now();
            
            match provider.speak(&task.request).await {
                Ok(()) => {
                    metrics.record_success(provider.name(), start_time.elapsed()).await;
                    return Ok(());
                }
                Err(e) => {
                    metrics.record_failure(provider.name(), &e).await;
                    tracing::warn!(
                        provider = provider.name(),
                        error = %e,
                        request_id = task.request.request_id,
                        "TTS provider failed, trying next"
                    );
                }
            }
        }
        
        Err(TtsError::AllProvidersFailed)
    }
}
```

#### 2.2 Provider Pool Management
```rust
/// Manages provider availability and health
pub struct ProviderPool {
    providers: Arc<RwLock<Vec<ProviderContainer>>>,
    health_monitor: Arc<TtsHealthMonitor>,
}

struct ProviderContainer {
    provider: Box<dyn TtsProvider>,
    status: ProviderStatus,
    last_health_check: DateTime<Utc>,
    consecutive_failures: u32,
}

#[derive(Debug, Clone)]
enum ProviderStatus {
    Available,
    Degraded { reason: String },
    Unavailable { until: DateTime<Utc> },
    Maintenance,
}
```

### 3. Memory Management and Performance

#### 3.1 Efficient Request Queuing
```rust
/// Lock-free queue for TTS requests
pub struct TtsRequestQueue {
    queue: crossbeam_queue::SegQueue<TtsTask>,
    max_size: usize,
    current_size: AtomicUsize,
    metrics: Arc<QueueMetrics>,
}

impl TtsRequestQueue {
    /// Non-blocking enqueue with backpressure
    pub fn try_enqueue(&self, task: TtsTask) -> Result<(), TtsTask> {
        if self.current_size.load(Ordering::Relaxed) >= self.max_size {
            return Err(task);
        }
        
        self.queue.push(task);
        self.current_size.fetch_add(1, Ordering::Relaxed);
        self.metrics.record_enqueue();
        Ok(())
    }
    
    /// Non-blocking dequeue
    pub fn try_dequeue(&self) -> Option<TtsTask> {
        if let Some(task) = self.queue.pop() {
            self.current_size.fetch_sub(1, Ordering::Relaxed);
            self.metrics.record_dequeue();
            Some(task)
        } else {
            None
        }
    }
}
```

#### 3.2 Resource Management
```rust
/// Resource management for TTS operations
pub struct TtsResourceManager {
    // Connection pools for HTTP clients
    elevenlabs_client: Option<Arc<ElevenLabsClient>>,
    openai_client: Option<Arc<OpenAiClient>>,
    
    // Audio processing resources
    audio_buffer_pool: ObjectPool<AudioBuffer>,
    
    // Rate limiters per provider
    rate_limiters: HashMap<String, RateLimiter>,
}

impl TtsResourceManager {
    /// Get or create HTTP client with connection pooling
    pub fn get_elevenlabs_client(&self) -> Option<Arc<ElevenLabsClient>> {
        self.elevenlabs_client.clone()
    }
    
    /// Acquire audio buffer from pool
    pub fn acquire_audio_buffer(&self) -> PooledBuffer {
        self.audio_buffer_pool.try_pull()
            .unwrap_or_else(|| AudioBuffer::new(DEFAULT_BUFFER_SIZE))
    }
}
```

## Dependencies & Constraints

### External Dependencies
- **tokio**: Async runtime and utilities (essential)
- **reqwest**: HTTP client for API providers (essential)
- **serde**: Configuration serialization (from PRD-01)
- **thiserror**: Error handling framework (from PRD-01)
- **tracing**: Structured logging (from PRD-01)
- **rodio**: Cross-platform audio playback (essential)
- **symphonia**: Audio format support (essential)

### Provider-Specific Dependencies
- **ElevenLabs**: HTTP client with multipart support
- **OpenAI**: HTTP client with streaming support
- **macOS**: System `say` command availability
- **pyttsx3**: Python runtime for fallback (compatibility)

### Technical Constraints
- **API Rate Limits**: ElevenLabs (25 req/min), OpenAI (varies by tier)
- **Audio Format Support**: MP3, WAV, OGG for cross-platform compatibility
- **Memory Budget**: <5MB for TTS subsystem including buffers
- **Latency Budget**: <1s from request to audio output
- **Platform Dependencies**: Native audio subsystem access required

### Design Constraints
- **Backward Compatibility**: Must support all existing Python configuration options
- **Non-blocking Operations**: All TTS operations must be async and non-blocking
- **Graceful Degradation**: System must continue functioning with reduced capability if TTS fails
- **Configuration Hot-reload**: Support runtime configuration updates without restart

## Success Criteria & Acceptance Tests

### Functional Success Criteria

1. **Provider Integration Completeness**
   - All four providers (ElevenLabs, OpenAI, macOS, pyttsx3) fully implemented
   - Fallback chain works correctly in all scenarios
   - Configuration compatibility with existing Python system
   - Voice selection and customization functional

2. **Notification System Integration**
   - Task completion announcements work correctly
   - User input request notifications functional
   - Error and security alert notifications delivered
   - Engineer name personalization working

3. **CLI Command Integration**
   - `maos notify` command functional with TTS
   - `maos stop` command stops TTS playback
   - TTS test and status commands working
   - Voice listing and selection functional

### Performance Success Criteria

1. **Latency Requirements**
   - Request queuing: <10ms
   - Provider fallback: <100ms
   - First audio output: <1s
   - Text processing: <50ms

2. **Throughput Requirements**
   - Support 10 concurrent TTS requests
   - Handle 100 requests/minute sustained load
   - Queue capacity of 50 pending requests
   - Provider health checks every 30 seconds

3. **Resource Efficiency**
   - Memory usage: <5MB steady state
   - CPU usage: <5% during TTS generation
   - Network connections: <5 persistent connections
   - Audio buffer usage optimized

### Quality Success Criteria

1. **Reliability Metrics**
   - 99.9% successful TTS delivery rate
   - <0.1% provider fallback failures
   - Zero memory leaks in 24-hour runs
   - Graceful handling of all error conditions

2. **Test Coverage**
   - >95% line coverage for all TTS code
   - Integration tests for all provider combinations
   - Load testing with sustained concurrent requests
   - Cross-platform compatibility validation

3. **Documentation and API**
   - 100% documented public APIs
   - Complete configuration reference
   - Troubleshooting guide for common issues
   - Performance tuning recommendations

## Testing Strategy

### 1. Unit Testing Approach
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    use mockall::predicate::*;
    
    #[tokio::test]
    async fn test_provider_fallback_chain() {
        let mut mock_elevenlabs = MockTtsProvider::new();
        mock_elevenlabs
            .expect_is_available()
            .returning(|| false);
        
        let mut mock_openai = MockTtsProvider::new();
        mock_openai
            .expect_is_available()
            .returning(|| true);
        mock_openai
            .expect_speak()
            .returning(|_| Ok(()));
        
        let manager = TtsManager::new_with_providers(vec![
            Box::new(mock_elevenlabs),
            Box::new(mock_openai),
        ]);
        
        let request = TtsRequest {
            text: "Test message".to_string(),
            voice: None,
            notification_type: NotificationType::TaskCompletion,
            urgency: NotificationUrgency::Normal,
            engineer_name: None,
            request_id: "test-123".to_string(),
            session_id: None,
        };
        
        assert!(manager.speak(request).await.is_ok());
    }
    
    #[tokio::test]
    async fn test_text_processing_performance() {
        let processor = TextProcessor::new(TextProcessingConfig::default());
        let long_text = "A".repeat(10000);
        
        let start = Instant::now();
        let processed = processor.process_for_speech(&long_text);
        let duration = start.elapsed();
        
        assert!(duration < Duration::from_millis(50));
        assert!(!processed.is_empty());
    }
}
```

### 2. Integration Testing
```rust
#[tokio::test]
async fn test_end_to_end_notification_flow() {
    let config = TtsConfig::default();
    let manager = TtsManager::new(config).await.unwrap();
    let notification_system = NotificationSystem::new(Arc::new(manager));
    
    // Test task completion notification
    let session_id = SessionId::generate();
    let agent_type: AgentType = "code-reviewer".to_string();
    
    let result = notification_system.announce_completion(
        &agent_type,
        &session_id,
        Some("Code review completed successfully"),
    ).await;
    
    assert!(result.is_ok());
    
    // Verify notification was logged
    // Verify TTS request was processed
    // Verify audio was generated (in test environment)
}
```

### 3. Load Testing
```rust
#[tokio::test]
async fn test_concurrent_tts_requests() {
    let manager = Arc::new(TtsManager::new(TtsConfig::default()).await.unwrap());
    let mut handles = Vec::new();
    
    // Submit 50 concurrent TTS requests
    for i in 0..50 {
        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let request = TtsRequest {
                text: format!("Test message {}", i),
                voice: None,
                notification_type: NotificationType::TaskCompletion,
                urgency: NotificationUrgency::Normal,
                engineer_name: None,
                request_id: format!("test-{}", i),
                session_id: None,
            };
            
            manager_clone.speak(request).await
        });
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    let results: Vec<_> = futures::future::join_all(handles).await;
    
    // Verify all requests succeeded
    for result in results {
        assert!(result.unwrap().is_ok());
    }
}
```

### 4. Cross-Platform Testing
- **Linux**: Test with various audio backends (ALSA, PulseAudio, JACK)
- **macOS**: Test with native Core Audio and various system voices
- **Windows**: Test with Windows Audio Session API and SAPI voices
- **Docker**: Test in containerized environments with limited audio access

## Timeline

### Week 1: Core Architecture and Provider Trait
**Days 1-2**: TTS provider trait design and core types
**Days 3-4**: TTS manager and async request processing pipeline
**Days 5-7**: Basic provider implementations (trait skeletons)

**Deliverables**:
- Complete provider trait definition
- TTS manager with async processing
- Basic error handling framework
- Unit tests for core architecture

### Week 2: Provider Implementations
**Days 1-2**: ElevenLabs provider with full API integration
**Days 3-4**: OpenAI provider with streaming support
**Days 5-7**: macOS and pyttsx3 providers with fallback logic

**Deliverables**:
- All four providers fully functional
- Provider health monitoring system
- Rate limiting and connection pooling
- Integration tests for each provider

### Week 3: Text Processing and Notification System
**Days 1-2**: Text processing pipeline with optimization
**Days 3-4**: Notification system integration with MAOS core
**Days 5-7**: CLI command integration and configuration management

**Deliverables**:
- Complete text processing system
- Notification system with template support
- CLI commands functional
- Configuration hot-reload support

### Week 4: Performance, Testing, and Documentation
**Days 1-2**: Performance optimization and benchmarking
**Days 3-4**: Comprehensive testing suite and cross-platform validation
**Days 5-7**: Documentation, examples, and integration validation

**Deliverables**:
- Performance targets achieved
- >95% test coverage
- Complete documentation and examples
- Production-ready TTS system

## Risk Assessment & Mitigation

### Technical Risks

**Risk**: Provider API changes break integration
**Probability**: Medium **Impact**: High
**Mitigation**: Version pinning, comprehensive integration tests, provider abstraction layer isolates changes

**Risk**: Audio playback issues on specific platforms
**Probability**: High **Impact**: Medium
**Mitigation**: Multiple audio backend support, fallback to system commands, extensive cross-platform testing

**Risk**: Performance targets not met due to audio processing overhead
**Probability**: Medium **Impact**: Medium
**Mitigation**: Async processing, efficient buffer management, benchmark-driven optimization

### Operational Risks

**Risk**: API quota exhaustion with paid providers
**Probability**: Medium **Impact**: Low
**Mitigation**: Built-in rate limiting, usage monitoring, automatic fallback to free providers

**Risk**: Network connectivity issues affecting cloud providers
**Probability**: High **Impact**: Low
**Mitigation**: Intelligent fallback to local providers, request caching, timeout handling

### Integration Risks

**Risk**: Breaking changes to existing notification interfaces
**Probability**: Low **Impact**: High
**Mitigation**: Backward compatibility testing, semantic versioning, migration guides

**Risk**: Configuration incompatibility with existing Python system
**Probability**: Medium **Impact**: Medium
**Mitigation**: Configuration validation, migration tooling, extensive compatibility testing

## Dependencies for Other PRDs

This TTS & Notifications PRD enables and integrates with:

### Direct Dependencies
- **PRD-01: Common Foundation** (uses core types, error handling, configuration)
- **PRD-02: Session Management** (integrates with session lifecycle notifications)
- **PRD-05: CLI Framework** (provides notify/stop commands)

### Integration Points
- **Security System**: TTS for security alerts and violations
- **Git Worktree Management**: Notifications for git operations and conflicts
- **Agent Orchestration**: Task completion and status announcements
- **Performance Monitoring**: TTS performance metrics integration

## Implementation Notes

### 1. Development Priority
This PRD has **P1 Priority** as it provides critical user feedback functionality that enhances the overall MAOS user experience. It can be developed in parallel with other systems but requires PRD-01 foundation.

### 2. Backward Compatibility
The Rust implementation must maintain 100% configuration compatibility with the existing Python system. A migration tool will assist users in transitioning from Python to Rust-based TTS.

### 3. Performance Monitoring
All TTS operations include comprehensive metrics collection to ensure performance targets are met and to identify optimization opportunities.

### 4. Extensibility
The provider trait system enables easy addition of new TTS providers (local AI models, cloud services) without changing core system architecture.

## Summary

The MAOS TTS & Notifications System represents a **complete modernization** of the existing Python TTS implementation, translating proven functionality into a high-performance Rust architecture. By maintaining full backward compatibility while adding enhanced reliability, async execution, and deep MAOS integration, this system will provide developers with **instant, reliable audio feedback** that enhances productivity and system awareness.

The multi-provider architecture with intelligent fallbacks ensures 99.9% delivery reliability while the async processing pipeline guarantees sub-second response times. Integration with MAOS core types and configuration provides a seamless developer experience that feels like a natural extension of the MAOS ecosystem.

**Expected Outcome**: A blazing-fast, rock-solid TTS system that makes MAOS feel more responsive and user-friendly while providing the foundation for advanced features like voice commands and real-time development narration. 🚀💯🎙️