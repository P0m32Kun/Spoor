use serde::{Deserialize, Serialize};

/// One row of scan output (see docs/INTEGRATION.md).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanResult {
    pub file: String,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    Path,
    Endpoint,
    Secret,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Origin {
    pub pattern: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EndpointParams {
    pub query: Vec<String>,
    pub body: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecretContext {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub nearby_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Finding {
    /// Source JS file (absolute path when available). Set at output time for JSONL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    pub kind: FindingKind,
    /// Primary payload: absolute URL (endpoint), secret string (secret), or path (path).
    pub value: String,
    /// Original endpoint string from source when `value` was resolved from a relative path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,
    pub confidence: Confidence,
    pub origin: Origin,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<EndpointParams>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<SecretContext>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
    /// HTTP status from live probe when `--from-url` verification is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,
}

impl Finding {
    pub fn path(value: impl Into<String>, origin: Origin) -> Self {
        Self {
            file: None,
            kind: FindingKind::Path,
            value: value.into(),
            raw: None,
            confidence: Confidence::Medium,
            origin,
            method: None,
            params: None,
            secret_type: None,
            severity: None,
            context: None,
            tags: Vec::new(),
            http_status: None,
        }
    }

    pub fn endpoint(value: impl Into<String>, method: impl Into<String>, origin: Origin) -> Self {
        Self {
            file: None,
            kind: FindingKind::Endpoint,
            value: value.into(),
            raw: None,
            confidence: Confidence::High,
            method: Some(method.into()),
            origin,
            params: None,
            secret_type: None,
            severity: None,
            context: None,
            tags: Vec::new(),
            http_status: None,
        }
    }

    pub fn secret(
        value: impl Into<String>,
        secret_type: impl Into<String>,
        severity: impl Into<String>,
        origin: Origin,
    ) -> Self {
        Self {
            file: None,
            kind: FindingKind::Secret,
            value: value.into(),
            raw: None,
            confidence: Confidence::High,
            origin,
            method: None,
            params: None,
            secret_type: Some(secret_type.into()),
            severity: Some(severity.into()),
            context: None,
            tags: Vec::new(),
            http_status: None,
        }
    }
}
