# Legacy System Integration Demo

This example demonstrates how to integrate multiple legacy data sources using Fluxion's **Wrapper Pattern** (Pattern 3 from the Integration Guide).

## Scenario

Aggregate data from 3 legacy systems that lack intrinsic timestamps:

1. **Legacy Database** - Emits JSON user records every 3 seconds
2. **Legacy Message Queue** - Emits XML order events every 2 seconds
3. **Legacy File Watcher** - Emits CSV inventory updates every 4 seconds

## Architecture

```
Legacy DB (JSON)          → User Adapter       → TimestampedUser ──────┐
                                                                       │
Legacy MQ (XML)           → Order Adapter      → TimestampedOrder      ├→ merge_with(Repository)
                                                                       │
Legacy File Watcher (CSV) → Inventory Adapter  → TimestampedInventory ─┘

                                                Repository State:
                                                - users: HashMap<UserId, User>
                                                - orders: HashMap<OrderId, Order>
                                                - inventory: HashMap<ProductId, Inventory>
```

## Key Concepts Demonstrated

### 1. Wrapper Pattern (Pattern 3)
Each adapter adds timestamps at the system boundary:

```rust
impl TimestampedUser {
    fn new(user: User) -> Self {
        let timestamp = chrono::Utc::now().timestamp_millis() as u64;
        Self { timestamp, user }
    }
}

impl Timestamped for TimestampedUser {
    type Inner = User;

    fn into_inner(self) -> Self::Inner {
        self.user
    }

    fn with_timestamp(user: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self { timestamp, user }
    }
}
```

### 2. merge_with for Stateful Aggregation
All three streams merge into a unified repository:

```rust
MergedStream::seed::<UnifiedEvent>(Repository::new())
    .merge_with(user_stream, |event, repo| {
        if let UnifiedEvent::UserAdded(user) = event {
            repo.users.insert(user.id, user);
        }
        event
    })
    .merge_with(order_stream, |event, repo| { /* ... */ })
    .merge_with(inventory_stream, |event, repo| { /* ... */ })
    .into_fluxion_stream()
```

### 3. Heterogeneous Data Sources
- Database returns JSON strings
- Message Queue returns XML messages
- File Watcher returns CSV lines

All unified into a single `UnifiedEvent` enum.

## Running the Demo

```bash
cd examples/legacy-integration
cargo run
```

Expected output:
```
🚀 Legacy Integration Demo Starting...

📊 Starting legacy data sources...
  🗄️  Legacy Database: Polling for new users (every 3s)
  📨 Legacy Message Queue: Consuming order events (every 2s)
  📁 Legacy File Watcher: Watching for CSV inventory files (every 4s)
🔄 Creating timestamped adapters...
🗄️  Building aggregated repository stream...

💼 Processing business logic...

Press Ctrl+C to stop...

================================================================================
✅ [0001] NEW USER: Alice Smith (alice.smith@legacy.com)
📦 [0002] NEW ORDER: #5000 - User 1001 wants 5 units of Product #101
📊 [0003] INVENTORY UPDATE: Widget B - 45 units available
...
================================================================================
✅ Demo completed successfully!
```

## Code Structure

```
src/
├── main.rs                     # Entry point, spawns legacy sources
├── domain/
│   ├── models.rs              # User, Order, Inventory structs
│   ├── events.rs              # UnifiedEvent enum
│   └── repository.rs          # Aggregated state + merge_with logic
├── legacy/
│   ├── database.rs            # Simulates polling a DB (JSON)
│   ├── message_queue.rs       # Simulates consuming from MQ (XML)
│   └── file_watcher.rs        # Simulates watching files (CSV)
├── adapters/
│   ├── user_adapter.rs        # Wraps User with timestamp
│   ├── order_adapter.rs       # Wraps Order with timestamp
│   └── inventory_adapter.rs   # Wraps Inventory with timestamp
└── processing/
    └── business_logic.rs      # Processes aggregated events
```

## Real-World Usage

In production, replace the simulators with actual integrations:

**Database:**
```rust
// Instead of simulated polling:
let users = sqlx::query_as::<_, User>("SELECT * FROM users WHERE processed = 0")
    .fetch_all(&pool)
    .await?;
```

**Message Queue:**
```rust
// Instead of simulated consumption:
let channel = connection.create_channel().await?;
channel.basic_consume("orders", "consumer", ...).await?;
```

**File Watcher:**
```rust
// Instead of simulated watching:
let watcher = notify::recommended_watcher(move |res| { /* ... */ })?;
watcher.watch("/var/legacy/inventory", RecursiveMode::NonRecursive)?;
```

## Benefits

✅ **Clean separation** - Legacy sources isolated from business logic
✅ **Temporal ordering** - Events processed in correct temporal sequence
✅ **Stateful aggregation** - Repository pattern with `merge_with`
✅ **Type safety** - Unified event model across heterogeneous sources
✅ **Production-ready** - Architecture scales to real integrations

## Next Steps

- Add error handling with the `on_error` operator
- Implement actual inventory checking before order fulfillment
- Add metrics and monitoring
- Persist repository state to disk
- Add more sophisticated business rules
