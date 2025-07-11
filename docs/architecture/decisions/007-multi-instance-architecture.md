# ADR-007: Multi-Instance Architecture

## Status
Accepted

## Context
MAOS, as an MCP server, needs to support multiple simultaneous instances running on the same machine. This is critical because:

- Users may have multiple IDEs/terminals hosting their own MCP server instances
- Different projects may need isolated MAOS instances
- Development and testing require running multiple instances
- Each instance needs its own port, lock file, and isolated state

Key requirements:
- Unique lock files per instance to prevent conflicts
- Port management for multiple HTTP/SSE servers
- Instance discovery and management
- Clean shutdown without affecting other instances

## Decision
We will implement a multi-instance architecture with automatic port allocation, unique lock files, and instance registration.

### Instance Identity and Lock Files

```rust
pub struct InstanceId {
    pub id: String,           // UUID for this instance
    pub pid: u32,            // Process ID
    pub port: u16,           // HTTP server port
    pub workspace_hash: String, // Project workspace identifier
    pub started_at: DateTime<Utc>,
}

impl InstanceManager {
    pub fn acquire_instance_lock(&self, workspace_path: &Path) -> Result<(InstanceId, PathBuf)> {
        let workspace_hash = hash_workspace_path(workspace_path);
        let instance_id = Uuid::new_v4().to_string();
        
        // Create unique lock file name
        let lock_file = format!("~/.maos/instances/{}-{}.lock", workspace_hash, instance_id);
        let lock_path = PathBuf::from(lock_file);
        
        // Try to create lock file exclusively
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)?;
            
        // Write instance info
        let instance = InstanceId {
            id: instance_id.clone(),
            pid: std::process::id(),
            port: 0, // Will be set after port allocation
            workspace_hash,
            started_at: Utc::now(),
        };
        
        file.write_all(serde_json::to_string(&instance)?.as_bytes())?;
        file.flush()?;
        
        // Lock the file (advisory lock)
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            fcntl::flock(file.as_raw_fd(), fcntl::FlockArg::LockExclusiveNonblock)?;
        }
        
        Ok((instance, lock_path))
    }
}
```

### Port Allocation Strategy

```rust
pub struct PortAllocator {
    base_port: u16,      // Default 3000
    max_port: u16,       // Default 3100
    preferred_ports: Vec<u16>, // User-configured preferences
}

impl PortAllocator {
    pub async fn allocate_port(&self, instance_id: &str) -> Result<u16> {
        // First try preferred ports
        for port in &self.preferred_ports {
            if self.is_port_available(*port).await? {
                return Ok(*port);
            }
        }
        
        // Then scan range for available port
        for port in self.base_port..=self.max_port {
            if self.is_port_available(port).await? {
                // Record port in instance registry
                self.register_port(instance_id, port).await?;
                return Ok(port);
            }
        }
        
        Err(anyhow!("No available ports in range {}-{}", self.base_port, self.max_port))
    }
    
    async fn is_port_available(&self, port: u16) -> Result<bool> {
        match TcpListener::bind(("127.0.0.1", port)).await {
            Ok(listener) => {
                drop(listener); // Release immediately
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }
}
```

### Instance Registry

Instance tracking is managed through the global instances database documented in the [Storage Schema Reference](../references/storage-schema.md#global-instance-database-schema). The schema includes fields for tracking instance ID, PID, port, workspace information, and health status.

### Instance Discovery

```rust
pub struct InstanceDiscovery {
    registry_db: SqliteConnection,
}

impl InstanceDiscovery {
    pub async fn list_instances(&self) -> Result<Vec<InstanceInfo>> {
        // Query all instances
        let instances = sqlx::query_as!(
            InstanceInfo,
            "SELECT * FROM instances WHERE status = 'running' ORDER BY started_at DESC"
        )
        .fetch_all(&self.registry_db)
        .await?;
        
        // Verify each instance is actually running
        let mut verified = Vec::new();
        for instance in instances {
            if self.verify_instance_alive(&instance).await {
                verified.push(instance);
            } else {
                // Mark as stopped
                self.mark_instance_stopped(&instance.id).await?;
            }
        }
        
        Ok(verified)
    }
    
    pub async fn find_instance_for_workspace(&self, workspace_path: &Path) -> Result<Option<InstanceInfo>> {
        let workspace_hash = hash_workspace_path(workspace_path);
        
        let instance = sqlx::query_as!(
            InstanceInfo,
            "SELECT * FROM instances WHERE workspace_hash = ? AND status = 'running' LIMIT 1",
            workspace_hash
        )
        .fetch_optional(&self.registry_db)
        .await?;
        
        // Verify it's actually running
        if let Some(inst) = instance {
            if self.verify_instance_alive(&inst).await {
                return Ok(Some(inst));
            }
        }
        
        Ok(None)
    }
    
    async fn verify_instance_alive(&self, instance: &InstanceInfo) -> bool {
        // Check if process exists
        #[cfg(unix)]
        {
            unsafe {
                libc::kill(instance.pid as i32, 0) == 0
            }
        }
        
        #[cfg(windows)]
        {
            // Windows-specific process check
            todo!()
        }
    }
}
```

### Health Monitoring

```rust
pub struct HealthMonitor {
    instance_id: String,
    registry_db: SqliteConnection,
    shutdown: Arc<AtomicBool>,
}

impl HealthMonitor {
    pub async fn start_heartbeat(&self) -> JoinHandle<()> {
        let instance_id = self.instance_id.clone();
        let db = self.registry_db.clone();
        let shutdown = self.shutdown.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            while !shutdown.load(Ordering::Relaxed) {
                interval.tick().await;
                
                // Update heartbeat
                if let Err(e) = sqlx::query!(
                    "UPDATE instances SET last_heartbeat = ? WHERE id = ?",
                    Utc::now(),
                    instance_id
                )
                .execute(&db)
                .await {
                    error!("Failed to update heartbeat: {}", e);
                }
            }
        })
    }
}
```

### Graceful Shutdown

```rust
impl MaosServer {
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down MAOS instance {}", self.instance_id);
        
        // 1. Stop accepting new connections
        self.shutdown_signal.store(true, Ordering::SeqCst);
        
        // 2. Shutdown all active agents
        self.process_manager.shutdown_all(Duration::from_secs(30)).await?;
        
        // 3. Update instance status
        sqlx::query!(
            "UPDATE instances SET status = 'stopped' WHERE id = ?",
            self.instance_id
        )
        .execute(&self.registry_db)
        .await?;
        
        // 4. Release lock file
        drop(self.lock_file);
        if let Err(e) = fs::remove_file(&self.lock_path) {
            warn!("Failed to remove lock file: {}", e);
        }
        
        // 5. Stop health monitor
        self.health_monitor.abort();
        
        Ok(())
    }
}
```

### MCP Client Configuration

Support for discovering existing instances:

```json
{
  "maos": {
    "transport": {
      "type": "http",
      "discovery": "automatic",  // or "port:3001" for specific instance
      "spawn_if_missing": true
    }
  }
}
```

### CLI Commands for Instance Management

```bash
# List all running instances
maos instances list

# Stop a specific instance
maos instances stop <instance-id>

# Clean up stale lock files
maos instances cleanup

# Show instance details
maos instances info <instance-id>
```

## Consequences

### Positive
- **Multiple Projects**: Each project can have its own MAOS instance
- **IDE Integration**: Multiple IDEs can run independent instances
- **Development**: Easy to test multi-instance scenarios
- **Robustness**: Crashed instances don't affect others
- **Discovery**: Clients can find existing instances automatically

### Negative
- **Resource Usage**: Each instance consumes memory and CPU
- **Port Conflicts**: Limited port range for instances
- **Complexity**: Lock file and registry management
- **Cleanup**: Stale instances need manual or automatic cleanup

### Mitigation
- Implement automatic cleanup of stale instances
- Make port range configurable
- Add instance pooling for resource efficiency
- Provide clear diagnostics for port conflicts

## Implementation Priority
1. Basic lock file mechanism
2. Port allocation
3. Instance registry
4. Health monitoring
5. Graceful shutdown
6. Discovery features
7. CLI management commands

## References
- [Storage Schema Reference](../references/storage-schema.md) - Instance database schema
- Unix advisory file locking (flock)
- TCP port binding and SO_REUSEADDR
- Process supervision patterns
- MCP transport discovery specification

---
*Date: 2025-07-09*  
*Author: Marvin (Claude)*  
*Reviewers: @clafollett (Cali LaFollett - LaFollett Labs LLC)*