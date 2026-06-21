use chrono::Utc;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use tokio::sync::RwLock;
use utoipa::{IntoParams, ToSchema};

type HmacSha256 = Hmac<Sha256>;
const GENESIS_SIGNATURE: &str = "0";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventCategory {
    Commitment,
    Swap,
    Admin,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct AuditEvent {
    pub sequence: u64,
    pub timestamp: i64,
    pub category: AuditEventCategory,
    pub event_type: String,
    pub actor_hash: Option<String>,
    pub ip_id: Option<u64>,
    pub swap_id: Option<u64>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub success: bool,
    pub status_code: Option<u16>,
    #[schema(value_type = Object)]
    pub details: Value,
    pub previous_signature: String,
    pub signature: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct UnsignedAuditEvent {
    sequence: u64,
    timestamp: i64,
    category: AuditEventCategory,
    event_type: String,
    actor_hash: Option<String>,
    ip_id: Option<u64>,
    swap_id: Option<u64>,
    request_id: Option<String>,
    trace_id: Option<String>,
    success: bool,
    status_code: Option<u16>,
    details: Value,
    previous_signature: String,
}

impl From<&AuditEvent> for UnsignedAuditEvent {
    fn from(event: &AuditEvent) -> Self {
        Self {
            sequence: event.sequence,
            timestamp: event.timestamp,
            category: event.category.clone(),
            event_type: event.event_type.clone(),
            actor_hash: event.actor_hash.clone(),
            ip_id: event.ip_id,
            swap_id: event.swap_id,
            request_id: event.request_id.clone(),
            trace_id: event.trace_id.clone(),
            success: event.success,
            status_code: event.status_code,
            details: event.details.clone(),
            previous_signature: event.previous_signature.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditEventInput {
    pub category: AuditEventCategory,
    pub event_type: String,
    pub actor_hash: Option<String>,
    pub ip_id: Option<u64>,
    pub swap_id: Option<u64>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub success: bool,
    pub status_code: Option<u16>,
    pub details: Value,
}

impl AuditEventInput {
    pub fn new(category: AuditEventCategory, event_type: impl Into<String>) -> Self {
        Self {
            category,
            event_type: event_type.into(),
            actor_hash: None,
            ip_id: None,
            swap_id: None,
            request_id: None,
            trace_id: None,
            success: true,
            status_code: None,
            details: Value::Object(Map::new()),
        }
    }

    pub fn actor_hash(mut self, actor_hash: Option<String>) -> Self {
        self.actor_hash = actor_hash.filter(|value| !value.is_empty());
        self
    }

    pub fn ip_id(mut self, ip_id: Option<u64>) -> Self {
        self.ip_id = ip_id;
        self
    }

    pub fn swap_id(mut self, swap_id: Option<u64>) -> Self {
        self.swap_id = swap_id;
        self
    }

    pub fn request_id(mut self, request_id: Option<String>) -> Self {
        self.request_id = request_id.filter(|value| !value.is_empty());
        self
    }

    pub fn trace_id(mut self, trace_id: Option<String>) -> Self {
        self.trace_id = trace_id.filter(|value| !value.is_empty());
        self
    }

    pub fn success(mut self, success: bool) -> Self {
        self.success = success;
        self
    }

    pub fn status_code(mut self, status_code: Option<u16>) -> Self {
        self.status_code = status_code;
        self
    }

    pub fn detail(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        let mut details = match self.details {
            Value::Object(map) => map,
            _ => Map::new(),
        };
        if let Ok(value) = serde_json::to_value(value) {
            details.insert(key.into(), value);
        }
        self.details = Value::Object(details);
        self
    }

    fn into_event(self, sequence: u64, previous_signature: String) -> AuditEvent {
        AuditEvent {
            sequence,
            timestamp: Utc::now().timestamp(),
            category: self.category,
            event_type: self.event_type,
            actor_hash: self.actor_hash,
            ip_id: self.ip_id,
            swap_id: self.swap_id,
            request_id: self.request_id,
            trace_id: self.trace_id,
            success: self.success,
            status_code: self.status_code,
            details: self.details,
            previous_signature,
            signature: String::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, IntoParams, ToSchema)]
pub struct AuditLogQuery {
    #[serde(default = "default_audit_limit")]
    pub limit: u64,
    #[serde(default)]
    pub offset: u64,
    pub category: Option<AuditEventCategory>,
    pub event_type: Option<String>,
    pub actor_hash: Option<String>,
    pub ip_id: Option<u64>,
    pub swap_id: Option<u64>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub success: Option<bool>,
    pub min_sequence: Option<u64>,
    pub max_sequence: Option<u64>,
}

impl Default for AuditLogQuery {
    fn default() -> Self {
        Self {
            limit: default_audit_limit(),
            offset: 0,
            category: None,
            event_type: None,
            actor_hash: None,
            ip_id: None,
            swap_id: None,
            request_id: None,
            trace_id: None,
            success: None,
            min_sequence: None,
            max_sequence: None,
        }
    }
}

fn default_audit_limit() -> u64 {
    50
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuditIntegrityStatus {
    Empty,
    Valid,
    Invalid,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct AuditLogQueryResponse {
    pub events: Vec<AuditEvent>,
    pub total_count: u64,
    pub has_more: bool,
    pub integrity: AuditIntegrityStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuditLogError {
    LockPoisoned,
    Serialization,
}

impl std::fmt::Display for AuditLogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LockPoisoned => formatter.write_str("audit log lock poisoned"),
            Self::Serialization => formatter.write_str("audit log serialization failed"),
        }
    }
}

impl std::error::Error for AuditLogError {}

pub struct AuditLogStore {
    key: Vec<u8>,
    events: RwLock<Vec<AuditEvent>>,
}

impl AuditLogStore {
    pub fn new(key: Vec<u8>) -> Self {
        let key = if key.is_empty() {
            rand::random::<[u8; 32]>().to_vec()
        } else {
            key
        };

        Self {
            key,
            events: RwLock::new(Vec::new()),
        }
    }

    pub fn from_env_or_random() -> Self {
        let key = std::env::var("AUDIT_HMAC_KEY")
            .ok()
            .filter(|value| !value.is_empty())
            .map(|value| value.into_bytes())
            .unwrap_or_else(|| rand::random::<[u8; 32]>().to_vec());

        Self::new(key)
    }

    pub fn hash_identifier(&self, identifier: &str) -> String {
        hmac_sha256_hex(&self.key, &format!("identifier:{identifier}"))
    }

    pub async fn append(&self, input: AuditEventInput) -> Result<AuditEvent, AuditLogError> {
        let (sequence, previous_signature) = {
            let events = self.events.read().await.map_err(|_| AuditLogError::LockPoisoned)?;
            let sequence = events.len() as u64 + 1;
            let previous_signature = events
                .last()
                .map(|event| event.signature.clone())
                .unwrap_or_else(|| GENESIS_SIGNATURE.to_string());
            (sequence, previous_signature)
        };

        let mut event = input.into_event(sequence, previous_signature);
        event.signature = compute_signature(&self.key, &event);

        self.events
            .write()
            .await
            .map_err(|_| AuditLogError::LockPoisoned)?
            .push(event.clone());

        Ok(event)
    }

    pub async fn query(&self, filter: &AuditLogQuery) -> AuditLogQueryResponse {
        let events = match self.events.read().await {
            Ok(events) => events,
            Err(_) => return AuditLogQueryResponse {
                events: Vec::new(),
                total_count: 0,
                has_more: false,
                integrity: AuditIntegrityStatus::Invalid,
            },
        };
        let limit = filter.limit.clamp(1, 200);
        let offset = filter.offset as usize;
        let filtered: Vec<AuditEvent> = events
            .iter()
            .filter(|event| event_matches_filter(event, filter))
            .cloned()
            .collect();
        let total_count = filtered.len() as u64;
        let page_end = offset.saturating_add(limit as usize).min(filtered.len());
        let page = filtered
            .iter()
            .skip(offset)
            .take(limit as usize)
            .cloned()
            .collect::<Vec<_>>();
        let has_more = page_end < filtered.len();
        let integrity = if events.is_empty() {
            AuditIntegrityStatus::Empty
        } else if verify_chain(&self.key, &events) {
            AuditIntegrityStatus::Valid
        } else {
            AuditIntegrityStatus::Invalid
        };

        AuditLogQueryResponse {
            events: page,
            total_count,
            has_more,
            integrity,
        }
    }

    pub async fn len(&self) -> usize {
        match self.events.read().await {
            Ok(events) => events.len(),
            Err(_) => 0,
        }
    }

    #[cfg(test)]
    pub async fn clear(&self) {
        self.events.write().await.unwrap().clear();
    }
}

fn event_matches_filter(event: &AuditEvent, filter: &AuditLogQuery) -> bool {
    if let Some(category) = &filter.category {
        if &event.category != category {
            return false;
        }
    }
    if let Some(event_type) = &filter.event_type {
        if &event.event_type != event_type {
            return false;
        }
    }
    if let Some(actor_hash) = &filter.actor_hash {
        if event.actor_hash.as_deref() != Some(actor_hash.as_str()) {
            return false;
        }
    }
    if let Some(ip_id) = filter.ip_id {
        if event.ip_id != Some(ip_id) {
            return false;
        }
    }
    if let Some(swap_id) = filter.swap_id {
        if event.swap_id != Some(swap_id) {
            return false;
        }
    }
    if let Some(request_id) = &filter.request_id {
        if event.request_id.as_deref() != Some(request_id.as_str()) {
            return false;
        }
    }
    if let Some(trace_id) = &filter.trace_id {
        if event.trace_id.as_deref() != Some(trace_id.as_str()) {
            return false;
        }
    }
    if let Some(success) = filter.success {
        if event.success != success {
            return false;
        }
    }
    if let Some(min_sequence) = filter.min_sequence {
        if event.sequence < min_sequence {
            return false;
        }
    }
    if let Some(max_sequence) = filter.max_sequence {
        if event.sequence > max_sequence {
            return false;
        }
    }

    true
}

fn compute_signature(key: &[u8], event: &AuditEvent) -> String {
    let unsigned = UnsignedAuditEvent::from(event);
    let message = serde_json::to_string(&unsigned).unwrap_or_else(|_| String::new());
    hmac_sha256_hex(key, &message)
}

fn verify_event_signature(key: &[u8], event: &AuditEvent) -> bool {
    let unsigned = UnsignedAuditEvent::from(event);
    let message = serde_json::to_string(&unsigned).unwrap_or_else(|_| String::new());
    let expected = hmac_sha256_hex(key, &message);
    constant_time_eq(event.signature.as_bytes(), expected.as_bytes())
}

fn verify_chain(key: &[u8], events: &[AuditEvent]) -> bool {
    let mut expected_previous = GENESIS_SIGNATURE.to_string();

    for (index, event) in events.iter().enumerate() {
        let expected_sequence = index as u64 + 1;
        if event.sequence != expected_sequence {
            return false;
        }
        if event.previous_signature != expected_previous {
            return false;
        }
        if !verify_event_signature(key, event) {
            return false;
        }
        expected_previous = event.signature.clone();
    }

    true
}

fn hmac_sha256_hex(key: &[u8], message: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts keys of any length");
    mac.update(message.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    left.iter()
        .zip(right.iter())
        .fold(0u8, |accumulator, (left, right)| accumulator | (left ^ right))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_append_creates_sequenced_hmac_signed_event() {
        let store = AuditLogStore::new(vec![7; 32]);
        let event = store
            .append(
                AuditEventInput::new(AuditEventCategory::Commitment, "ip.commit.requested")
                    .ip_id(Some(1))
                    .detail("commitment_hash", "deadbeef"),
            )
            .await
            .unwrap();

        assert_eq!(event.sequence, 1);
        assert_eq!(event.previous_signature, GENESIS_SIGNATURE);
        assert!(!event.signature.is_empty());
        assert_eq!(event.details["commitment_hash"], "deadbeef");
    }

    #[tokio::test]
    async fn test_query_filters_by_category_ip_and_success() {
        let store = AuditLogStore::new(vec![9; 32]);
        let actor_hash = store.hash_identifier("GOWNER");
        store
            .append(
                AuditEventInput::new(AuditEventCategory::Commitment, "ip.commit.requested")
                    .actor_hash(Some(actor_hash.clone()))
                    .ip_id(Some(1))
                    .success(false),
            )
            .await
            .unwrap();
        store
            .append(
                AuditEventInput::new(AuditEventCategory::Swap, "swap.accept.requested")
                    .swap_id(Some(2))
                    .success(true),
            )
            .await
            .unwrap();

        let response = store
            .query(&AuditLogQuery {
                category: Some(AuditEventCategory::Commitment),
                ip_id: Some(1),
                success: Some(false),
                ..AuditLogQuery::default()
            })
            .await;

        assert_eq!(response.total_count, 1);
        assert_eq!(response.events.len(), 1);
        assert_eq!(response.events[0].event_type, "ip.commit.requested");
        assert_eq!(response.events[0].actor_hash.as_deref(), Some(actor_hash.as_str()));
        assert_eq!(response.integrity, AuditIntegrityStatus::Valid);
    }

    #[tokio::test]
    async fn test_query_limits_and_reports_has_more() {
        let store = AuditLogStore::new(vec![11; 32]);
        store
            .append(AuditEventInput::new(AuditEventCategory::Admin, "admin.action"))
            .await
            .unwrap();
        store
            .append(AuditEventInput::new(AuditEventCategory::Admin, "admin.action"))
            .await
            .unwrap();

        let response = store
            .query(&AuditLogQuery {
                limit: 1,
                offset: 0,
                ..AuditLogQuery::default()
            })
            .await;

        assert_eq!(response.total_count, 2);
        assert_eq!(response.events.len(), 1);
        assert!(response.has_more);
    }

    #[tokio::test]
    async fn test_tamper_detection_finds_modified_details() {
        let store = AuditLogStore::new(vec![13; 32]);
        let mut event = store
            .append(
                AuditEventInput::new(AuditEventCategory::Admin, "admin.action")
                    .detail("action", "register"),
            )
            .await
            .unwrap();

        if let Some(details) = event.details.as_object_mut() {
            details.insert("action".to_string(), json!("tamper"));
        }
        let events = vec![event];

        assert!(!verify_chain(store.key.as_slice(), &events));
    }

    #[tokio::test]
    async fn test_tamper_detection_finds_sequence_gap() {
        let store = AuditLogStore::new(vec![17; 32]);
        let mut first = store
            .append(AuditEventInput::new(AuditEventCategory::Admin, "admin.action"))
            .await
            .unwrap();
        let second = store
            .append(AuditEventInput::new(AuditEventCategory::Admin, "admin.action"))
            .await
            .unwrap();

        first.sequence = 99;
        let events = vec![first, second];

        assert!(!verify_chain(store.key.as_slice(), &events));
    }

    #[tokio::test]
    async fn test_hash_identifier_is_stable_without_exposing_input() {
        let store = AuditLogStore::new(vec![19; 32]);
        let hashed = store.hash_identifier("GOWNER@example.invalid");

        assert_eq!(hashed.len(), 64);
        assert!(!hashed.contains("GOWNER"));
        assert_eq!(hashed, store.hash_identifier("GOWNER@example.invalid"));
        assert_ne!(hashed, store.hash_identifier("GOTHER@example.invalid"));
    }
}
