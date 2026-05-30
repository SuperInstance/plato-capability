use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single capability descriptor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Capability {
    pub name: String,
    pub version: String,
    pub description: String,
    pub parameters: HashMap<String, String>,
}

impl Capability {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: String::new(),
            parameters: HashMap::new(),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
}

/// A set of capabilities.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CapabilitySet {
    capabilities: HashMap<String, Capability>,
}

impl CapabilitySet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, cap: Capability) {
        self.capabilities.insert(cap.name.clone(), cap);
    }

    pub fn has(&self, name: &str) -> bool {
        self.capabilities.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&Capability> {
        self.capabilities.get(name)
    }

    pub fn satisfies(&self, req: &CapabilityRequirement) -> bool {
        match self.get(&req.capability) {
            Some(cap) => version_compatible(&req.min_version, &cap.version),
            None => false,
        }
    }

    pub fn missing(&self, requirements: &[CapabilityRequirement]) -> Vec<String> {
        requirements
            .iter()
            .filter(|req| !self.satisfies(req))
            .map(|req| req.capability.clone())
            .collect()
    }

    pub fn len(&self) -> usize {
        self.capabilities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.capabilities.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Capability> {
        self.capabilities.values()
    }
}

/// A requirement for a specific capability.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilityRequirement {
    pub capability: String,
    pub min_version: String,
    pub required: bool,
}

impl CapabilityRequirement {
    pub fn new(capability: impl Into<String>, min_version: impl Into<String>, required: bool) -> Self {
        Self {
            capability: capability.into(),
            min_version: min_version.into(),
            required,
        }
    }

    pub fn required(capability: impl Into<String>, min_version: impl Into<String>) -> Self {
        Self::new(capability, min_version, true)
    }

    pub fn optional(capability: impl Into<String>, min_version: impl Into<String>) -> Self {
        Self::new(capability, min_version, false)
    }
}

/// Result of capability negotiation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NegotiationResult {
    Compatible,
    PartiallyCompatible(Vec<String>),
    Incompatible(Vec<String>),
}

/// Describes an agent and its capabilities.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentDescriptor {
    pub agent_id: String,
    pub capabilities: CapabilitySet,
    pub metadata: HashMap<String, String>,
}

impl AgentDescriptor {
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            capabilities: CapabilitySet::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_capabilities(mut self, caps: CapabilitySet) -> Self {
        self.capabilities = caps;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Describes a room and its capabilities.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoomDescriptor {
    pub room_id: String,
    pub room_type: String,
    pub capabilities: CapabilitySet,
    pub sensors: Vec<String>,
}

impl RoomDescriptor {
    pub fn new(room_id: impl Into<String>, room_type: impl Into<String>) -> Self {
        Self {
            room_id: room_id.into(),
            room_type: room_type.into(),
            capabilities: CapabilitySet::new(),
            sensors: Vec::new(),
        }
    }

    pub fn with_capabilities(mut self, caps: CapabilitySet) -> Self {
        self.capabilities = caps;
        self
    }

    pub fn with_sensor(mut self, sensor: impl Into<String>) -> Self {
        self.sensors.push(sensor.into());
        self
    }
}

/// Perform capability negotiation between an agent and a room.
///
/// The room's capabilities must satisfy the agent's required capabilities.
/// Returns:
/// - Compatible: all requirements met
/// - PartiallyCompatible: all required met, some optional missing
/// - Incompatible: some required capabilities missing
pub fn negotiate(agent: &AgentDescriptor, room: &RoomDescriptor) -> NegotiationResult {
    let mut req_missing: Vec<String> = Vec::new();

    // We need the actual requirements. Since AgentDescriptor doesn't store requirements directly,
    // we treat the agent's capabilities as the requirements.
    // Each agent capability is treated as a required capability that the room must satisfy.
    for cap in agent.capabilities.iter() {
        let req = CapabilityRequirement::required(&cap.name, &cap.version);
        if !room.capabilities.satisfies(&req) {
            req_missing.push(cap.name.clone());
        }
    }

    if req_missing.is_empty() {
        NegotiationResult::Compatible
    } else {
        NegotiationResult::Incompatible(req_missing)
    }
}


/// Check if an available version satisfies a required minimum version.
///
/// Simple semver-like check: parses major.minor.patch and compares numerically.
/// Missing components default to 0.
pub fn version_compatible(required: &str, available: &str) -> bool {
    let req_parts = parse_version(required);
    let avail_parts = parse_version(available);

    for i in 0..3 {
        if avail_parts[i] < req_parts[i] {
            return false;
        }
        if avail_parts[i] > req_parts[i] {
            return true;
        }
    }
    true // exact match
}

fn parse_version(v: &str) -> [u32; 3] {
    let parts: Vec<&str> = v.trim_start_matches('v').split('.').collect();
    let mut result = [0u32; 3];
    for i in 0..3 {
        if let Some(s) = parts.get(i) {
            result[i] = s.parse().unwrap_or(0);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_creation() {
        let cap = Capability::new("audio-input", "1.2.0")
            .with_description("Microphone input")
            .with_parameter("channels", "2");
        assert_eq!(cap.name, "audio-input");
        assert_eq!(cap.version, "1.2.0");
        assert_eq!(cap.description, "Microphone input");
        assert_eq!(cap.parameters.get("channels"), Some(&"2".to_string()));
    }

    #[test]
    fn capability_serialization() {
        let cap = Capability::new("video", "2.0.0");
        let json = serde_json::to_string(&cap).unwrap();
        let back: Capability = serde_json::from_str(&json).unwrap();
        assert_eq!(cap, back);
    }

    #[test]
    fn capability_set_add_has_get() {
        let mut set = CapabilitySet::new();
        assert!(!set.has("audio"));
        assert!(set.get("audio").is_none());

        set.add(Capability::new("audio", "1.0.0"));
        assert!(set.has("audio"));
        assert_eq!(set.get("audio").unwrap().version, "1.0.0");
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn capability_set_overwrite() {
        let mut set = CapabilitySet::new();
        set.add(Capability::new("audio", "1.0.0"));
        set.add(Capability::new("audio", "2.0.0"));
        assert_eq!(set.get("audio").unwrap().version, "2.0.0");
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn version_compatible_exact() {
        assert!(version_compatible("1.0.0", "1.0.0"));
    }

    #[test]
    fn version_compatible_higher() {
        assert!(version_compatible("1.0.0", "2.0.0"));
        assert!(version_compatible("1.2.0", "1.3.0"));
        assert!(version_compatible("1.0.0", "1.0.1"));
    }

    #[test]
    fn version_compatible_lower() {
        assert!(!version_compatible("2.0.0", "1.9.9"));
        assert!(!version_compatible("1.1.0", "1.0.9"));
    }

    #[test]
    fn version_compatible_partial_version() {
        assert!(version_compatible("1", "1.0.0"));
        assert!(version_compatible("1.0", "1.0.0"));
    }

    #[test]
    fn satisfies_full_match() {
        let mut set = CapabilitySet::new();
        set.add(Capability::new("audio", "2.1.0"));
        let req = CapabilityRequirement::required("audio", "2.0.0");
        assert!(set.satisfies(&req));
    }

    #[test]
    fn satisfies_version_too_low() {
        let mut set = CapabilitySet::new();
        set.add(Capability::new("audio", "1.0.0"));
        let req = CapabilityRequirement::required("audio", "2.0.0");
        assert!(!set.satisfies(&req));
    }

    #[test]
    fn satisfies_missing_capability() {
        let set = CapabilitySet::new();
        let req = CapabilityRequirement::required("audio", "1.0.0");
        assert!(!set.satisfies(&req));
    }

    #[test]
    fn missing_detects_missing() {
        let mut set = CapabilitySet::new();
        set.add(Capability::new("audio", "1.0.0"));
        let reqs = vec![
            CapabilityRequirement::required("audio", "1.0.0"),
            CapabilityRequirement::required("video", "1.0.0"),
        ];
        let missing = set.missing(&reqs);
        assert_eq!(missing, vec!["video"]);
    }

    #[test]
    fn missing_none() {
        let mut set = CapabilitySet::new();
        set.add(Capability::new("audio", "2.0.0"));
        let reqs = vec![CapabilityRequirement::required("audio", "1.0.0")];
        assert!(set.missing(&reqs).is_empty());
    }

    #[test]
    fn negotiate_compatible() {
        let mut agent_caps = CapabilitySet::new();
        agent_caps.add(Capability::new("audio", "1.0.0"));

        let mut room_caps = CapabilitySet::new();
        room_caps.add(Capability::new("audio", "2.0.0"));

        let agent = AgentDescriptor::new("agent-1").with_capabilities(agent_caps);
        let room = RoomDescriptor::new("room-1", "living").with_capabilities(room_caps);

        assert_eq!(negotiate(&agent, &room), NegotiationResult::Compatible);
    }

    #[test]
    fn negotiate_incompatible() {
        let mut agent_caps = CapabilitySet::new();
        agent_caps.add(Capability::new("audio", "2.0.0"));

        let mut room_caps = CapabilitySet::new();
        room_caps.add(Capability::new("audio", "1.0.0"));

        let agent = AgentDescriptor::new("agent-1").with_capabilities(agent_caps);
        let room = RoomDescriptor::new("room-1", "living").with_capabilities(room_caps);

        match negotiate(&agent, &room) {
            NegotiationResult::Incompatible(missing) => {
                assert!(missing.contains(&"audio".to_string()));
            }
            _ => panic!("Expected Incompatible"),
        }
    }

    #[test]
    fn negotiate_empty_agent() {
        let agent = AgentDescriptor::new("agent-1");
        let room = RoomDescriptor::new("room-1", "living");
        assert_eq!(negotiate(&agent, &room), NegotiationResult::Compatible);
    }

    #[test]
    fn negotiate_missing_capability_in_room() {
        let mut agent_caps = CapabilitySet::new();
        agent_caps.add(Capability::new("video", "1.0.0"));

        let room = RoomDescriptor::new("room-1", "living");
        let agent = AgentDescriptor::new("agent-1").with_capabilities(agent_caps);

        match negotiate(&agent, &room) {
            NegotiationResult::Incompatible(missing) => {
                assert_eq!(missing, vec!["video"]);
            }
            _ => panic!("Expected Incompatible"),
        }
    }

    #[test]
    fn agent_descriptor_builder() {
        let agent = AgentDescriptor::new("agent-42")
            .with_metadata("role", "assistant");
        assert_eq!(agent.agent_id, "agent-42");
        assert_eq!(agent.metadata.get("role"), Some(&"assistant".to_string()));
    }

    #[test]
    fn room_descriptor_builder() {
        let mut caps = CapabilitySet::new();
        caps.add(Capability::new("audio", "1.0.0"));
        let room = RoomDescriptor::new("room-1", "kitchen")
            .with_capabilities(caps)
            .with_sensor("thermometer")
            .with_sensor("microphone");
        assert_eq!(room.room_id, "room-1");
        assert_eq!(room.room_type, "kitchen");
        assert_eq!(room.sensors, vec!["thermometer", "microphone"]);
    }

    #[test]
    fn empty_capability_set() {
        let set = CapabilitySet::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn capability_requirement_optional() {
        let req = CapabilityRequirement::optional("audio", "1.0.0");
        assert!(!req.required);
    }

    #[test]
    fn negotiation_result_serialization() {
        let result = NegotiationResult::PartiallyCompatible(vec!["video".to_string()]);
        let json = serde_json::to_string(&result).unwrap();
        let back: NegotiationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, back);
    }
}
