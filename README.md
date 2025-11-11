# fluxion

100% Rust implementation of composite Rx extensions for stream aggregation.

## Workspace Structure

This workspace contains three crates:

### **fluxion-stream**
Stream combinators with ordering guarantees for async Rust.
- `combine_latest` - Combine multiple streams, emitting when all have values
- `combine_with_previous` - Pair each value with the previous one
- `with_latest_from` - Combine stream with latest value from another
- `merge_with` - Merge streams with custom state management
- `take_latest_when` - Take latest value when condition is met
- `select_all_ordered` - Select from multiple streams with ordering
- `sequenced` - Core sequencing types for temporal ordering
- `sequenced_channel` - Channels that auto-sequence messages

### **fluxion-exec**
Async stream subscribers and execution utilities.
- `subscribe_async` - Sequential async processing of stream items
- `subscribe_latest_async` - Parallel async processing with cancellation

### **fluxion-test-utils**
Shared test infrastructure and utilities.
- Test data models (Person, Animal, Plant, SimpleEnum)
- Test helpers and assertions
- Common test infrastructure

## Status

✅ All 40 tests passing
✅ Workspace restructured for modularity
🚧 POC still ongoing - not yet published to crates.io
