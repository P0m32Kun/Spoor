//! Optional parity checks against [jsluice](https://github.com/BishopFox/jsluice).
//!
//! Tests skip automatically when `jsluice` is not installed.
//! Install: `go install github.com/BishopFox/jsluice/cmd/jsluice@latest`

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use serde::Deserialize;

    use crate::analyzer::Analyzer;
    use crate::finding::FindingKind;

    #[derive(Debug, Deserialize)]
    struct JsluiceUrlRow {
        url: String,
        #[serde(rename = "type")]
        row_type: String,
    }

    #[derive(Debug, Deserialize)]
    struct JsluiceSecretRow {
        data: serde_json::Value,
    }

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root")
    }

    fn fixture_path(rel: &str) -> PathBuf {
        repo_root().join(rel)
    }

    fn jsluice_bin() -> Option<PathBuf> {
        if command_ok("jsluice") {
            return Some(PathBuf::from("jsluice"));
        }
        if let Ok(home) = std::env::var("HOME") {
            let candidate = PathBuf::from(home).join("go/bin/jsluice");
            if candidate.is_file() && command_ok(candidate.to_str()?) {
                return Some(candidate);
            }
        }
        None
    }

    fn command_ok(bin: &str) -> bool {
        Command::new(bin)
            .arg("--help")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn run_jsluice_urls(path: &Path, bin: &Path) -> Vec<JsluiceUrlRow> {
        let output = Command::new(bin)
            .args(["urls", path.to_str().expect("utf-8 path")])
            .output()
            .expect("spawn jsluice urls");
        assert!(
            output.status.success(),
            "jsluice urls failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        parse_json_lines(&output.stdout)
    }

    fn run_jsluice_secrets(path: &Path, bin: &Path) -> Vec<JsluiceSecretRow> {
        let output = Command::new(bin)
            .args(["secrets", path.to_str().expect("utf-8 path")])
            .output()
            .expect("spawn jsluice secrets");
        assert!(
            output.status.success(),
            "jsluice secrets failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        parse_json_lines(&output.stdout)
    }

    fn parse_json_lines<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Vec<T> {
        String::from_utf8_lossy(bytes)
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| serde_json::from_str(l).expect("jsluice json line"))
            .collect()
    }

    fn jsluice_semantic_urls(rows: &[JsluiceUrlRow]) -> HashSet<String> {
        rows.iter()
            .filter(|r| r.row_type != "stringLiteral")
            .map(|r| r.url.clone())
            .collect()
    }

    fn jsluice_string_literal_urls(rows: &[JsluiceUrlRow]) -> HashSet<String> {
        rows.iter()
            .filter(|r| r.row_type == "stringLiteral")
            .map(|r| r.url.clone())
            .collect()
    }

    fn spoor_url_sets(source: &str, filename: &str) -> (HashSet<String>, HashSet<String>) {
        let findings = Analyzer::new(source, Some(filename)).collect_findings();
        let endpoints = findings
            .iter()
            .filter(|f| f.kind == FindingKind::Endpoint)
            .map(|f| f.value.clone())
            .collect();
        let paths = findings
            .iter()
            .filter(|f| f.kind == FindingKind::Path)
            .map(|f| f.value.clone())
            .collect();
        (endpoints, paths)
    }

    fn spoor_secret_values(source: &str, filename: &str) -> HashSet<String> {
        Analyzer::new(source, Some(filename))
            .collect_findings()
            .into_iter()
            .filter(|f| f.kind == FindingKind::Secret)
            .map(|f| f.value)
            .collect()
    }

    fn jsluice_secret_values(rows: &[JsluiceSecretRow]) -> HashSet<String> {
        rows.iter()
            .filter_map(|r| r.data.get("key").and_then(|k| k.as_str()))
            .map(str::to_string)
            .collect()
    }

    fn read_fixture(rel: &str) -> (PathBuf, String) {
        let path = fixture_path(rel);
        let source = std::fs::read_to_string(&path).expect("read fixture");
        (path, source)
    }

    struct UrlParityCase {
        fixture: &'static str,
        filename: &'static str,
    }

    const URL_PARITY_FIXTURES: &[UrlParityCase] = &[
        UrlParityCase {
            fixture: "tests/fixtures/jsluice_subset.js",
            filename: "jsluice_subset.js",
        },
        UrlParityCase {
            fixture: "tests/fixtures/phase1/combined.js",
            filename: "combined.js",
        },
        UrlParityCase {
            fixture: "tests/fixtures/katana/spa_bundle.js",
            filename: "spa_bundle.js",
        },
        UrlParityCase {
            fixture: "tests/fixtures/katana/legacy_admin.js",
            filename: "legacy_admin.js",
        },
        UrlParityCase {
            fixture: "tests/fixtures/katana/api_clients.js",
            filename: "api_clients.js",
        },
    ];

    #[test]
    fn jsluice_semantic_urls_are_found_by_spoor() {
        let Some(bin) = jsluice_bin() else {
            eprintln!("skip jsluice_semantic_urls_are_found_by_spoor: jsluice not in PATH");
            return;
        };

        for case in URL_PARITY_FIXTURES {
            let (path, source) = read_fixture(case.fixture);
            let j_rows = run_jsluice_urls(&path, &bin);
            let j_semantic = jsluice_semantic_urls(&j_rows);
            let (spoor_apis, spoor_paths) = spoor_url_sets(&source, case.filename);
            let spoor_all = spoor_apis
                .iter()
                .chain(spoor_paths.iter())
                .cloned()
                .collect::<HashSet<_>>();

            for url in &j_semantic {
                assert!(
                    spoor_apis.contains(url),
                    "{}: jsluice semantic url {url:?} not in spoor apis.\n  spoor apis: {spoor_apis:?}\n  jsluice types: {:?}",
                    case.filename,
                    j_rows
                        .iter()
                        .filter(|r| r.url == *url && r.row_type != "stringLiteral")
                        .map(|r| &r.row_type)
                        .collect::<Vec<_>>()
                );
            }

            // jsluice stringLiteral rows should still appear somewhere in Spoor output
            for url in jsluice_string_literal_urls(&j_rows) {
                if j_semantic.contains(&url) {
                    continue;
                }
                assert!(
                    spoor_all.contains(&url),
                    "{}: jsluice stringLiteral url {url:?} missing from spoor paths/apis",
                    case.filename
                );
            }
        }
    }

    #[test]
    fn jsluice_secret_keys_are_found_by_spoor() {
        let Some(bin) = jsluice_bin() else {
            eprintln!("skip jsluice_secret_keys_are_found_by_spoor: jsluice not in PATH");
            return;
        };

        for (fixture, filename) in [
            ("tests/fixtures/secrets.js", "secrets.js"),
            ("tests/fixtures/katana/secrets_leak.js", "secrets_leak.js"),
            ("tests/fixtures/katana/spa_bundle.js", "spa_bundle.js"),
        ] {
            let (path, source) = read_fixture(fixture);
            let j_rows = run_jsluice_secrets(&path, &bin);
            let j_keys = jsluice_secret_values(&j_rows);
            let spoor_secrets = spoor_secret_values(&source, filename);

            for key in &j_keys {
                assert!(
                    spoor_secrets.contains(key),
                    "{filename}: jsluice secret key {key:?} not found in spoor secrets.\n  spoor: {spoor_secrets:?}"
                );
            }
        }
    }

    /// Spoor intentionally omits bogus XHR `.open` on non-XHR objects (jsluice still matches).
    #[test]
    fn spoor_stricter_than_jsluice_on_fake_xhr_open() {
        let Some(bin) = jsluice_bin() else {
            eprintln!("skip spoor_stricter_than_jsluice_on_fake_xhr_open: jsluice not in PATH");
            return;
        };

        let (path, source) = read_fixture("tests/fixtures/katana/negative_controls.js");
        let j_rows = run_jsluice_urls(&path, &bin);
        let j_semantic = jsluice_semantic_urls(&j_rows);
        assert!(
            j_semantic.contains("/not-an-xhr-endpoint"),
            "expected jsluice false positive on fake xhr.open"
        );

        let (spoor_apis, _) = spoor_url_sets(&source, "negative_controls.js");
        assert!(
            !spoor_apis.contains("/not-an-xhr-endpoint"),
            "spoor must not treat db.open as xhr endpoint"
        );
    }

    /// Document Spoor extensions beyond jsluice semantic URL set (informational guard).
    #[test]
    fn spoor_extensions_beyond_jsluice_on_api_clients() {
        let Some(bin) = jsluice_bin() else {
            eprintln!("skip spoor_extensions_beyond_jsluice_on_api_clients: jsluice not in PATH");
            return;
        };

        let (path, source) = read_fixture("tests/fixtures/katana/api_clients.js");
        let j_semantic = jsluice_semantic_urls(&run_jsluice_urls(&path, &bin));
        let (spoor_apis, _) = spoor_url_sets(&source, "api_clients.js");

        let extensions: HashSet<_> = spoor_apis.difference(&j_semantic).cloned().collect();
        assert!(
            extensions.contains("wss://realtime.example.com/ws"),
            "expected WebSocket extension, got {extensions:?}"
        );
        assert!(
            extensions.contains("/api/v1/ws"),
            "expected relative WebSocket extension, got {extensions:?}"
        );
    }
}
