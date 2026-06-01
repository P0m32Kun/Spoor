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
    pub kind: FindingKind,
    pub value: String,
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
}

impl Finding {
    pub fn path(value: impl Into<String>, origin: Origin) -> Self {
        Self {
            kind: FindingKind::Path,
            value: value.into(),
            confidence: Confidence::Medium,
            origin,
            method: None,
            params: None,
            secret_type: None,
            severity: None,
            context: None,
            tags: Vec::new(),
        }
    }

    pub fn endpoint(value: impl Into<String>, method: impl Into<String>, origin: Origin) -> Self {
        Self {
            kind: FindingKind::Endpoint,
            value: value.into(),
            confidence: Confidence::High,
            method: Some(method.into()),
            origin,
            params: None,
            secret_type: None,
            severity: None,
            context: None,
            tags: Vec::new(),
        }
    }

    pub fn secret(
        value: impl Into<String>,
        secret_type: impl Into<String>,
        severity: impl Into<String>,
        origin: Origin,
    ) -> Self {
        Self {
            kind: FindingKind::Secret,
            value: value.into(),
            confidence: Confidence::High,
            origin,
            method: None,
            params: None,
            secret_type: Some(secret_type.into()),
            severity: Some(severity.into()),
            context: None,
            tags: Vec::new(),
        }
    }
}
