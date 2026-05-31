//! Spoor core library — parse JS with Oxc and extract recon findings.

mod analyzer;
mod dedup;
mod finding;
mod matcher;
mod string_fold;
mod url;

pub use analyzer::{Analyzer, ParseOutcome};
pub use finding::{
    Confidence, EndpointParams, Finding, FindingKind, Origin, ScanResult, SecretContext,
};
pub use string_fold::collapsed_string;
pub use url::maybe_url;
