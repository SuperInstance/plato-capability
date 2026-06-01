# plato-capability

> Capability descriptors and negotiation for PLATO rooms and agents

## What This Does

plato-capability provides a system for describing what PLATO rooms and agents can do, then negotiating whether they're compatible. Each room advertises capabilities with version numbers. Each agent declares what it requires. The negotiation engine checks version compatibility and reports missing capabilities.

## The Key Idea

Before an agent can work with a room, it needs to know what the room supports. A room might have "audio-input v2.0" and "temperature-sensor v1.5". An agent might require "audio-input v1.0+". The negotiation checks semver-like compatibility (major.minor.patch) and returns Compatible, PartiallyCompatible, or Incompatible with a list of what's missing.

## Install

```bash
cargo add plato-capability
```

## Quick Start

```rust
use plato_capability::*;

// Room advertises capabilities
let mut room_caps = CapabilitySet::new();
room_caps.add(Capability::new("audio-input", "2.1.0").with_description("Microphone"));

// Agent requires capabilities
let mut agent_caps = CapabilitySet::new();
agent_caps.add(Capability::new("audio-input", "1.0.0"));

let agent = AgentDescriptor::new("agent-1").with_capabilities(agent_caps);
let room = RoomDescriptor::new("kitchen", "room").with_capabilities(room_caps);

match negotiate(&agent, &room) {
    NegotiationResult::Compatible => println!("Good to go!"),
    NegotiationResult::Incompatible(missing) => println!("Missing: {:?}", missing),
    _ => {}
}
```

## API Reference

| Type | Description |
|---|---|
| `Capability { name, version, description, parameters }` | A single capability with semver version |
| `CapabilitySet` | Named capability collection. `add()`, `has()`, `get()`, `satisfies(req)`, `missing(reqs)` |
| `CapabilityRequirement { capability, min_version, required }` | What's needed. `required()` / `optional()` constructors. |
| `AgentDescriptor { agent_id, capabilities, metadata }` | An agent and what it requires |
| `RoomDescriptor { room_id, room_type, capabilities, sensors }` | A room and what it provides |
| `NegotiationResult` | `Compatible` / `PartiallyCompatible(missing)` / `Incompatible(missing)` |
| `version_compatible(required, available) -> bool` | Semver check: major.minor.patch |

## Testing

22 tests: capability creation, set operations, version compatibility (exact/higher/lower/partial), satisfaction, missing detection, negotiation (compatible/incompatible/empty), agent/room builders, serialization.

## License

Apache-2.0
