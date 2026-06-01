//! Spoor core library — parse JS with Oxc and extract recon findings.

mod analyzer;
mod dedup;
mod finding;
mod http_status;
mod matcher;
mod output;
mod secret_patterns;
mod secret_text;
mod string_fold;
mod url;

#[cfg(test)]
mod katana_regression;
#[cfg(test)]
mod jsluice_parity;

pub use analyzer::{Analyzer, ParseOutcome};
pub use finding::{
    Confidence, EndpointParams, Finding, FindingKind, Origin, ScanResult, SecretContext,
};
pub use http_status::{is_acceptable_http_status, is_http_url};
pub use output::{prepare_for_output, OutputOptions};
pub use secret_patterns::is_js_resource_url;
pub use secret_text::collect_secrets_from_document;
pub use string_fold::collapsed_string;
pub use url::{infer_base_url, is_absolute_url, maybe_url, origin_from_urlish, resolve_endpoint_url, resolved_maybe_url};
