# MAOS Coordination Mechanisms Design

## Overview

MAOS coordination enables multiple Claude Code agents to work together efficiently through file-based communication, lock management, and progress tracking. The system is designed for high concurrency, conflict prevention, and seamless collaboration.

## Architecture

### Coordination Components

```rust
maos-coordination/
├── src/
│   ├── lib.rs                  // Public API
│   ├── communication/
│   │   ├── mod.rs              // Communication patterns
│   │   ├── channels.rs         // File-based channels
│   │   ├── messages.rs         // Message types
│   │   ├── broadcast.rs        // Broadcast messaging
│   │   └── pubsub.rs           // Pub/sub implementation
│   ├── locking/
│   │   ├── mod.rs              // Lock management
│   │   ├── file_lock.rs        // File-based locks
│   │   ├── distributed.rs      // Distributed locking
│   │   ├── advisory.rs         // Advisory locks
│   │   └── deadlock.rs         // Deadlock detection
│   ├── tracking/
│   │   ├── mod.rs              // Progress tracking
│   │   ├── tasks.rs            // Task management
│   │   ├── dependencies.rs     // Task dependencies
│   │   └── visualization.rs    // Progress visualization
│   └── merge/
│       ├── mod.rs              // Merge coordination
│       ├── conflicts.rs        // Conflict detection
│       ├── strategies.rs       // Merge strategies
│       └── automation.rs       // Automated merging
```

## File-Based Communication

### Message Channel Design

```rust
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use tokio::fs;
use tokio::sync::watch;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FileChannel {
    base_path: PathBuf,
    channel_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message<T> {
    pub id: Uuid,
    pub timestamp: u64,
    pub sender: AgentId,
    pub recipients: Recipients,
    pub payload: T,
    pub metadata: MessageMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Recipients {
    All,
    Agent(AgentId),
    Group(Vec<AgentId>),
    Role(AgentRole),
}

impl FileChannel {
    pub fn new(workspace: &Path, channel_name: &str) -> Self {
        let base_path = workspace
            .join(".maos")
            .join("channels")
            .join(channel_name);
        
        Self {
            base_path,
            channel_id: channel_name.to_string(),
        }
    }
    
    pub async fn send<T: Serialize>(&self, message: Message<T>) -> Result<()> {
        // Ensure channel directory exists
        fs::create_dir_all(&self.base_path).await?;
        
        // Write message to timestamped file
        let filename = format!("{:020}_{}.json", message.timestamp, message.id);
        let path = self.base_path.join(filename);
        
        let content = serde_json::to_string_pretty(&message)?;
        fs::write(&path, content).await?;
        
        // Write marker for watchers
        let marker_path = self.base_path.join(".latest");
        fs::write(&marker_path, message.id.to_string()).await?;
        
        Ok(())
    }
    
    pub async fn receive<T: DeserializeOwned>(&self, after: Option<u64>) -> Result<Vec<Message<T>>> {
        let mut messages = Vec::new();
        let mut entries = fs::read_dir(&self.base_path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension() == Some("json".as_ref()) {
                let filename = path.file_name().unwrap().to_str().unwrap();
                
                // Parse timestamp from filename
                if let Some(timestamp_str) = filename.split('_').next() {
                    if let Ok(timestamp) = timestamp_str.parse::<u64>() {
                        if after.map_or(true, |t| timestamp > t) {
                            let content = fs::read_to_string(&path).await?;
                            if let Ok(message) = serde_json::from_str(&content) {
                                messages.push(message);
                            }
                        }
                    }
                }
            }
        }
        
        messages.sort_by_key(|m| m.timestamp);
        Ok(messages)
    }
}
```

### Broadcast Communication

```rust
pub struct BroadcastChannel {
    channels: Arc<RwLock<HashMap<String, FileChannel>>>,
    workspace: PathBuf,
}

impl BroadcastChannel {
    pub async fn broadcast<T: Serialize + Clone>(
        &self,
        topic: &str,
        message: T,
    ) -> Result<()> {
        let broadcast_msg = Message {
            id: Uuid::new_v4(),
            timestamp: current_timestamp(),
            sender: AgentId::current(),
            recipients: Recipients::All,
            payload: message,
            metadata: Default::default(),
        };
        
        // Write to topic directory
        let topic_channel = FileChannel::new(&self.workspace, &format!("broadcast/{}", topic));
        topic_channel.send(broadcast_msg).await?;
        
        // Also write to global broadcast
        let global_channel = FileChannel::new(&self.workspace, "broadcast/global");
        global_channel.send(Message {
            payload: BroadcastNotification {
                topic: topic.to_string(),
                timestamp: current_timestamp(),
            },
            ..Default::default()
        }).await?;
        
        Ok(())
    }
}
```

### Watch-Based Real-Time Updates

```rust
use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};
use std::sync::mpsc::channel;
use std::time::Duration;

pub struct ChannelWatcher {
    channel: FileChannel,
    tx: watch::Sender<Option<Uuid>>,
    rx: watch::Receiver<Option<Uuid>>,
}

impl ChannelWatcher {
    pub fn new(channel: FileChannel) -> Result<Self> {
        let (tx, rx) = watch::channel(None);
        let watcher_tx = tx.clone();
        let marker_path = channel.base_path.join(".latest");
        
        // Create file watcher
        let (file_tx, file_rx) = channel();
        let mut watcher = watcher(file_tx, Duration::from_millis(100))?;
        watcher.watch(&marker_path, RecursiveMode::NonRecursive)?;
        
        // Spawn watcher thread
        tokio::spawn(async move {
            loop {
                match file_rx.recv() {
                    Ok(DebouncedEvent::Write(_)) | Ok(DebouncedEvent::Create(_)) => {
                        if let Ok(content) = fs::read_to_string(&marker_path).await {
                            if let Ok(id) = Uuid::parse_str(&content) {
                                watcher_tx.send(Some(id)).ok();
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
        
        Ok(Self { channel, tx, rx })
    }
    
    pub async fn wait_for_message<T: DeserializeOwned>(&mut self) -> Result<Message<T>> {
        loop {
            self.rx.changed().await?;
            if let Some(id) = *self.rx.borrow() {
                // Read new messages
                let messages = self.channel.receive::<T>(None).await?;
                if let Some(msg) = messages.into_iter().find(|m| m.id == id) {
                    return Ok(msg);
                }
            }
        }
    }
}
```

## Lock Management

### File-Based Distributed Locking

```rust
use std::time::{Duration, SystemTime};
use tokio::time::sleep;

#[derive(Debug, Serialize, Deserialize)]
pub struct LockInfo {
    pub owner: AgentId,
    pub acquired_at: SystemTime,
    pub expires_at: SystemTime,
    pub purpose: String,
    pub metadata: HashMap<String, String>,
}

pub struct FileLock {
    path: PathBuf,
    lock_file: PathBuf,
    timeout: Duration,
}

impl FileLock {
    pub fn new(resource_path: &Path, timeout: Duration) -> Self {
        let lock_file = resource_path.with_extension("lock");
        Self {
            path: resource_path.to_path_buf(),
            lock_file,
            timeout,
        }
    }
    
    pub async fn acquire(&self, purpose: &str) -> Result<LockGuard> {
        let start = SystemTime::now();
        let max_wait = Duration::from_secs(30);
        
        loop {
            // Try to create lock file atomically
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&self.lock_file)
                .await
            {
                Ok(mut file) => {
                    // Write lock info
                    let lock_info = LockInfo {
                        owner: AgentId::current(),
                        acquired_at: SystemTime::now(),
                        expires_at: SystemTime::now() + self.timeout,
                        purpose: purpose.to_string(),
                        metadata: HashMap::new(),
                    };
                    
                    let content = serde_json::to_string(&lock_info)?;
                    file.write_all(content.as_bytes()).await?;
                    file.sync_all().await?;
                    
                    return Ok(LockGuard {
                        lock: self.clone(),
                        info: lock_info,
                    });
                }
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                    // Check if lock is expired
                    if let Ok(content) = fs::read_to_string(&self.lock_file).await {
                        if let Ok(info) = serde_json::from_str::<LockInfo>(&content) {
                            if SystemTime::now() > info.expires_at {
                                // Lock expired, try to remove
                                fs::remove_file(&self.lock_file).await.ok();
                                continue;
                            }
                        }
                    }
                    
                    // Check timeout
                    if SystemTime::now().duration_since(start)? > max_wait {
                        return Err(anyhow!("Lock acquisition timeout"));
                    }
                    
                    // Wait and retry
                    sleep(Duration::from_millis(100)).await;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
    
    pub async fn try_acquire(&self, purpose: &str) -> Result<Option<LockGuard>> {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.lock_file)
            .await
        {
            Ok(mut file) => {
                let lock_info = LockInfo {
                    owner: AgentId::current(),
                    acquired_at: SystemTime::now(),
                    expires_at: SystemTime::now() + self.timeout,
                    purpose: purpose.to_string(),
                    metadata: HashMap::new(),
                };
                
                let content = serde_json::to_string(&lock_info)?;
                file.write_all(content.as_bytes()).await?;
                
                Ok(Some(LockGuard {
                    lock: self.clone(),
                    info: lock_info,
                }))
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

pub struct LockGuard {
    lock: FileLock,
    info: LockInfo,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        // Release lock by removing file
        tokio::spawn({
            let lock_file = self.lock.lock_file.clone();
            async move {
                fs::remove_file(lock_file).await.ok();
            }
        });
    }
}
```

### Advisory Locking for Cooperation

```rust
pub struct AdvisoryLock {
    registry: Arc<DashMap<String, LockInfo>>,
    workspace: PathBuf,
}

impl AdvisoryLock {
    pub async fn request_lock(&self, resource: &str, purpose: &str) -> Result<bool> {
        let lock_path = self.workspace
            .join(".maos")
            .join("locks")
            .join("advisory")
            .join(format!("{}.lock", resource));
        
        // Check if anyone else is interested
        let interests_path = lock_path.with_extension("interests");
        let mut interests = self.read_interests(&interests_path).await?;
        
        // Add our interest
        interests.insert(AgentId::current(), Interest {
            purpose: purpose.to_string(),
            timestamp: SystemTime::now(),
            priority: self.calculate_priority(purpose),
        });
        
        // Write interests
        self.write_interests(&interests_path, &interests).await?;
        
        // Wait for our turn based on priority
        self.wait_for_turn(&interests, &lock_path).await
    }
}
```

### Deadlock Detection

```rust
pub struct DeadlockDetector {
    dependency_graph: Arc<RwLock<DiGraph<AgentId, LockDependency>>>,
    detection_interval: Duration,
}

impl DeadlockDetector {
    pub async fn start_monitoring(self: Arc<Self>) {
        let mut interval = tokio::time::interval(self.detection_interval);
        
        loop {
            interval.tick().await;
            
            if let Some(cycle) = self.detect_cycle().await {
                self.handle_deadlock(cycle).await;
            }
        }
    }
    
    async fn detect_cycle(&self) -> Option<Vec<AgentId>> {
        let graph = self.dependency_graph.read().await;
        
        // Use Tarjan's algorithm for cycle detection
        let scc = tarjan_scc(&*graph);
        
        // Find cycles (SCCs with more than one node)
        scc.into_iter()
            .find(|component| component.len() > 1)
    }
    
    async fn handle_deadlock(&self, cycle: Vec<AgentId>) {
        // Notify affected agents
        let notification = DeadlockNotification {
            detected_at: SystemTime::now(),
            involved_agents: cycle.clone(),
            suggested_resolution: self.suggest_resolution(&cycle).await,
        };
        
        // Broadcast deadlock detection
        let broadcast = BroadcastChannel::new(&self.workspace);
        broadcast.broadcast("deadlock", notification).await.ok();
    }
}
```

## Progress Tracking

### Task Management System

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub name: String,
    pub description: String,
    pub status: TaskStatus,
    pub assigned_to: Option<AgentId>,
    pub dependencies: Vec<TaskId>,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub metadata: TaskMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Blocked,
    InProgress,
    Review,
    Completed,
    Failed,
}

pub struct TaskTracker {
    tasks: Arc<DashMap<TaskId, Task>>,
    workspace: PathBuf,
}

impl TaskTracker {
    pub async fn create_task(&self, task: Task) -> Result<TaskId> {
        let id = task.id.clone();
        
        // Validate dependencies exist
        for dep_id in &task.dependencies {
            if !self.tasks.contains_key(dep_id) {
                return Err(anyhow!("Dependency {} not found", dep_id));
            }
        }
        
        // Store task
        self.tasks.insert(id.clone(), task.clone());
        
        // Persist to file
        self.persist_task(&task).await?;
        
        // Notify subscribers
        self.notify_task_created(&task).await?;
        
        Ok(id)
    }
    
    pub async fn update_status(
        &self,
        task_id: &TaskId,
        new_status: TaskStatus,
    ) -> Result<()> {
        let mut task = self.tasks.get_mut(task_id)
            .ok_or_else(|| anyhow!("Task not found"))?;
        
        let old_status = task.status;
        task.status = new_status;
        task.updated_at = SystemTime::now();
        
        // Check if this unblocks other tasks
        if new_status == TaskStatus::Completed {
            self.check_unblocked_tasks(task_id).await?;
        }
        
        // Persist and notify
        self.persist_task(&*task).await?;
        self.notify_status_change(task_id, old_status, new_status).await?;
        
        Ok(())
    }
    
    async fn check_unblocked_tasks(&self, completed_id: &TaskId) -> Result<()> {
        for entry in self.tasks.iter() {
            let (id, task) = entry.pair();
            
            if task.status == TaskStatus::Blocked && 
               task.dependencies.contains(completed_id) {
                // Check if all dependencies are completed
                let all_complete = task.dependencies.iter().all(|dep| {
                    self.tasks.get(dep)
                        .map(|t| t.status == TaskStatus::Completed)
                        .unwrap_or(false)
                });
                
                if all_complete {
                    self.update_status(id, TaskStatus::Pending).await?;
                }
            }
        }
        
        Ok(())
    }
}
```

### Progress Visualization

```rust
pub struct ProgressVisualizer {
    tracker: Arc<TaskTracker>,
    output_path: PathBuf,
}

impl ProgressVisualizer {
    pub async fn generate_progress_report(&self) -> Result<ProgressReport> {
        let tasks: Vec<_> = self.tracker.tasks.iter()
            .map(|entry| entry.value().clone())
            .collect();
        
        let total = tasks.len();
        let completed = tasks.iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();
        let in_progress = tasks.iter()
            .filter(|t| t.status == TaskStatus::InProgress)
            .count();
        let blocked = tasks.iter()
            .filter(|t| t.status == TaskStatus::Blocked)
            .count();
        
        let report = ProgressReport {
            timestamp: SystemTime::now(),
            total_tasks: total,
            completed_tasks: completed,
            in_progress_tasks: in_progress,
            blocked_tasks: blocked,
            completion_percentage: (completed as f32 / total as f32) * 100.0,
            task_breakdown: self.generate_breakdown(&tasks),
            critical_path: self.calculate_critical_path(&tasks).await?,
        };
        
        // Write report
        let content = serde_json::to_string_pretty(&report)?;
        fs::write(&self.output_path, content).await?;
        
        Ok(report)
    }
    
    pub async fn generate_gantt_chart(&self) -> Result<String> {
        // Generate Mermaid gantt chart
        let mut chart = String::from("gantt\n");
        chart.push_str("    title Task Progress\n");
        chart.push_str("    dateFormat YYYY-MM-DD HH:mm\n");
        
        for entry in self.tracker.tasks.iter() {
            let task = entry.value();
            let status = match task.status {
                TaskStatus::Completed => "done",
                TaskStatus::InProgress => "active",
                _ => "pending",
            };
            
            chart.push_str(&format!(
                "    {} :{}, {}, {}\n",
                task.name,
                status,
                format_time(task.created_at),
                format_duration(task.estimated_duration)
            ));
        }
        
        Ok(chart)
    }
}
```

## Merge Coordination

### Conflict Detection

```rust
pub struct MergeCoordinator {
    workspace: PathBuf,
    conflict_detector: ConflictDetector,
}

impl MergeCoordinator {
    pub async fn prepare_merge(
        &self,
        source_branch: &str,
        target_branch: &str,
    ) -> Result<MergePreparation> {
        // Acquire merge lock
        let lock = FileLock::new(
            &self.workspace.join(".maos").join("merge.lock"),
            Duration::from_secs(300),
        );
        let _guard = lock.acquire("merge preparation").await?;
        
        // Detect potential conflicts
        let conflicts = self.conflict_detector
            .detect_conflicts(source_branch, target_branch)
            .await?;
        
        // Analyze affected files
        let affected_files = self.get_affected_files(source_branch, target_branch).await?;
        
        // Check which agents modified files
        let agent_modifications = self.analyze_agent_modifications(&affected_files).await?;
        
        Ok(MergePreparation {
            source: source_branch.to_string(),
            target: target_branch.to_string(),
            conflicts,
            affected_files,
            agent_modifications,
            suggested_strategy: self.suggest_merge_strategy(&conflicts),
        })
    }
    
    pub async fn coordinate_resolution(
        &self,
        merge_prep: &MergePreparation,
    ) -> Result<Resolution> {
        // Notify affected agents
        for (agent_id, files) in &merge_prep.agent_modifications {
            self.notify_agent_of_conflicts(agent_id, files).await?;
        }
        
        // Create resolution tasks
        let mut tasks = Vec::new();
        for conflict in &merge_prep.conflicts {
            let task = Task {
                id: TaskId::new(),
                name: format!("Resolve conflict in {}", conflict.file_path),
                description: conflict.description.clone(),
                status: TaskStatus::Pending,
                assigned_to: conflict.suggested_resolver.clone(),
                dependencies: vec![],
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
                metadata: Default::default(),
            };
            tasks.push(task);
        }
        
        // Track resolution progress
        let resolution_tracker = ResolutionTracker::new(tasks);
        resolution_tracker.start_tracking().await?;
        
        Ok(Resolution {
            id: Uuid::new_v4(),
            merge_preparation: merge_prep.clone(),
            tracker: resolution_tracker,
        })
    }
}

pub struct ConflictDetector {
    git_repo: Repository,
}

impl ConflictDetector {
    pub async fn detect_conflicts(
        &self,
        source: &str,
        target: &str,
    ) -> Result<Vec<Conflict>> {
        // Use git to detect conflicts
        let merge_base = self.find_merge_base(source, target)?;
        let source_changes = self.get_changes_since(source, &merge_base)?;
        let target_changes = self.get_changes_since(target, &merge_base)?;
        
        let mut conflicts = Vec::new();
        
        // Find overlapping changes
        for (file, source_change) in &source_changes {
            if let Some(target_change) = target_changes.get(file) {
                if self.changes_conflict(source_change, target_change) {
                    conflicts.push(Conflict {
                        file_path: file.clone(),
                        source_change: source_change.clone(),
                        target_change: target_change.clone(),
                        description: self.describe_conflict(source_change, target_change),
                        suggested_resolver: self.suggest_resolver(file),
                    });
                }
            }
        }
        
        Ok(conflicts)
    }
}
```

## Integration Examples

### Agent Communication Example

```rust
// In an agent implementation
pub async fn coordinate_with_backend(api_spec: &ApiSpec) -> Result<()> {
    let channel = FileChannel::new(&workspace(), "backend-frontend");
    
    let message = Message {
        id: Uuid::new_v4(),
        timestamp: current_timestamp(),
        sender: AgentId::current(),
        recipients: Recipients::Role(AgentRole::Backend),
        payload: FrontendRequest::ImplementApi {
            spec: api_spec.clone(),
            priority: Priority::High,
        },
        metadata: Default::default(),
    };
    
    channel.send(message).await?;
    
    // Wait for response
    let mut watcher = ChannelWatcher::new(channel)?;
    let response: Message<BackendResponse> = watcher.wait_for_message().await?;
    
    match response.payload {
        BackendResponse::ApiImplemented { endpoints } => {
            println!("Backend implemented {} endpoints", endpoints.len());
        }
        BackendResponse::Error { reason } => {
            return Err(anyhow!("Backend error: {}", reason));
        }
    }
    
    Ok(())
}
```

### Task Tracking Example

```rust
// In orchestrator agent
pub async fn track_feature_implementation(feature: &Feature) -> Result<()> {
    let tracker = TaskTracker::new(workspace());
    
    // Create main task
    let main_task = Task {
        id: TaskId::new(),
        name: format!("Implement {}", feature.name),
        description: feature.description.clone(),
        status: TaskStatus::Pending,
        assigned_to: None,
        dependencies: vec![],
        created_at: SystemTime::now(),
        updated_at: SystemTime::now(),
        metadata: Default::default(),
    };
    
    let main_id = tracker.create_task(main_task).await?;
    
    // Create subtasks
    let design_id = tracker.create_task(Task {
        name: "Design architecture".to_string(),
        assigned_to: Some(AgentId::from("architect")),
        ..Default::default()
    }).await?;
    
    let backend_id = tracker.create_task(Task {
        name: "Implement backend".to_string(),
        assigned_to: Some(AgentId::from("backend-engineer")),
        dependencies: vec![design_id.clone()],
        ..Default::default()
    }).await?;
    
    let frontend_id = tracker.create_task(Task {
        name: "Implement frontend".to_string(),
        assigned_to: Some(AgentId::from("frontend-engineer")),
        dependencies: vec![design_id.clone()],
        ..Default::default()
    }).await?;
    
    // Monitor progress
    let visualizer = ProgressVisualizer::new(tracker.clone());
    
    loop {
        let report = visualizer.generate_progress_report().await?;
        println!("Progress: {:.1}% complete", report.completion_percentage);
        
        if report.completed_tasks == report.total_tasks {
            break;
        }
        
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
    
    Ok(())
}
```

This coordination system provides robust mechanisms for multi-agent collaboration while maintaining simplicity and performance.