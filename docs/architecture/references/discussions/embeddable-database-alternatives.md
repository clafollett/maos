# Embeddable Database Alternatives Discussion

## The Problem
Docker deployment adds complexity:
- Requires Docker installation
- Complex volume mounting for CLI access
- Harder to debug and troubleshoot
- Barriers to adoption for many users

## Requirements Recap
We need a database that supports:
1. **Concurrent writes** from multiple agent processes
2. **Event sourcing** with JSON/structured data
3. **Pub/sub or notifications** for inter-process communication
4. **ACID transactions** for consistency
5. **Good query capabilities** for event replay
6. **Embedded deployment** (no separate server process)

## Option 1: DuckDB
Modern columnar database designed for analytics, but embeddable.

**Pros:**
- Excellent concurrent read performance
- Native JSON support
- SQL interface
- Single file deployment
- Great for event sourcing (append-only workloads)
- Written in C++ with Rust bindings

**Cons:**
- Optimized for OLAP, not OLTP
- No built-in pub/sub
- Relatively new (but stable)

**Usage Pattern:**
```rust
use duckdb::{Connection, Result};

let conn = Connection::open("maos.duckdb")?;

// Event sourcing table
conn.execute(
    "CREATE TABLE events (
        id BIGINT PRIMARY KEY,
        aggregate_id UUID,
        event_data JSON,
        occurred_at TIMESTAMP
    )",
    [],
)?;
```

## Option 2: RocksDB + Custom Event Store
Facebook's embeddable key-value store.

**Pros:**
- Battle-tested at scale
- Excellent write performance
- Supports concurrent access
- Column families for different data types
- Used by TiKV, CockroachDB

**Cons:**
- Key-value only (no SQL)
- Must build event sourcing on top
- No built-in pub/sub

**Usage Pattern:**
```rust
use rocksdb::{DB, Options};

let path = "maos-rocksdb";
let mut opts = Options::default();
opts.create_if_missing(true);
opts.increase_parallelism(num_cpus::get() as i32);

let db = DB::open(&opts, path)?;

// Store events with sortable keys
let key = format!("events:{}:{}", aggregate_id, version);
db.put(key, serde_json::to_vec(&event)?)?;
```

## Option 3: Sled
Modern embedded database written in Rust.

**Pros:**
- Pure Rust
- Lock-free concurrent B+ tree
- ACID transactions
- Async subscriptions (pub/sub!)
- Simple API

**Cons:**
- Still beta (but widely used)
- No SQL interface
- Limited query capabilities

**Usage Pattern:**
```rust
use sled::{Db, Event};

let db = sled::open("maos-db")?;

// Event sourcing
let events = db.open_tree("events")?;
events.insert(key, bincode::serialize(&event)?)?;

// Pub/sub via subscriptions
let subscriber = events.watch_prefix(b"task:");
while let Some(event) = (&mut subscriber).await {
    match event {
        Event::Insert { key, value } => {
            // Handle new task
        }
    }
}
```

## Option 4: SQLite with Write-Ahead Logging (WAL)
Revisiting SQLite with better concurrency settings.

**Pros:**
- Ubiquitous, stable, well-understood
- WAL mode improves concurrency
- JSON1 extension for JSONB-like features
- Single file deployment

**Cons:**
- Still has write serialization
- No built-in pub/sub
- Limited concurrent writes

**Improvements over original approach:**
```rust
let conn = Connection::open("maos.db")?;

// Enable WAL mode for better concurrency
conn.execute("PRAGMA journal_mode=WAL", [])?;
conn.execute("PRAGMA synchronous=NORMAL", [])?;
conn.execute("PRAGMA busy_timeout=5000", [])?;

// Use IMMEDIATE transactions to reduce conflicts
let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
```

## Option 5: Hybrid Approach - File System + Locking
Following claude-code-flow's pattern.

**Architecture:**
```
maos-data/
├── events/
│   ├── 2024/01/01/
│   │   ├── event-001.json
│   │   └── event-002.json
│   └── index.json
├── sessions/
│   └── {session-id}/
│       ├── metadata.json
│       └── state.json
├── queues/
│   └── tasks/
│       ├── pending/
│       └── completed/
└── locks/
    └── .lock-files
```

**Pros:**
- Zero dependencies
- Easy to debug (just files)
- Natural partitioning
- Can use file system notifications

**Cons:**
- Complex locking required
- Performance concerns at scale
- Must implement indexing

## Option 6: Turso (libSQL)
SQLite fork designed for edge/embedded with better concurrency.

**Pros:**
- SQLite compatible
- Better concurrent write support
- Embedded mode available
- Native replication support

**Cons:**
- Newer project
- Less battle-tested

## Recommendation: Sled + Message Passing

**Why Sled:**
1. **Pure Rust** - No C dependencies
2. **Built-in subscriptions** - Solves our pub/sub needs
3. **Concurrent writes** - Lock-free design
4. **Simple deployment** - Just a directory

**Architecture:**
```rust
pub struct MaosStorage {
    // Core trees
    events: sled::Tree,        // Event sourcing
    sessions: sled::Tree,      // Session state
    agents: sled::Tree,        // Agent registry
    
    // Message passing via Sled subscriptions
    task_queue: sled::Tree,    // Task assignments
    status_updates: sled::Tree, // Progress updates
}

impl MaosStorage {
    pub async fn watch_task_assignments(&self) -> Subscriber {
        self.task_queue.watch_prefix(b"new:")
    }
    
    pub async fn publish_task(&self, task: Task) -> Result<()> {
        let key = format!("new:{}", task.id);
        self.task_queue.insert(key, bincode::serialize(&task)?)?;
        Ok(())
    }
}
```

**Deployment becomes:**
```bash
# Just run the binary!
./maos

# Or with systemd
systemctl start maos

# No Docker required!
```

## Migration Path

If we start with Sled and need to migrate later:
1. Export events to standard format
2. Import into PostgreSQL/other database
3. Repository pattern makes this seamless

## Decision Factors

Choose **PostgreSQL + Docker** if:
- Need production-grade SQL capabilities
- Want proven scalability
- Have Docker expertise

Choose **Sled** if:
- Want simple deployment
- Need embedded solution
- Prefer pure Rust stack
- Want built-in pub/sub

Choose **File System** if:
- Ultra-simple deployment critical
- Want maximum transparency
- OK with complexity trade-off