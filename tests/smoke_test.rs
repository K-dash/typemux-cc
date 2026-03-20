mod support;

use support::{PackageConfig, ProxyUnderTest, WorkspaceConfig};

/// Priority 1: Basic LSP lifecycle — initialize → initialized → shutdown → exit.
///
/// Verifies that the proxy can complete the full lifecycle with a single
/// mock backend and exit cleanly.
#[tokio::test]
async fn smoke_test_lifecycle() {
    let scenario = serde_json::json!({
        "on_startup": [],
        "steps": [
            {
                "expect": { "method": "initialize" },
                "actions": [{
                    "type": "respond",
                    "body": {
                        "capabilities": {
                            "textDocumentSync": 1,
                            "hoverProvider": true
                        }
                    }
                }]
            },
            {
                "expect": { "method": "initialized" },
                "actions": []
            },
            {
                "expect": { "method": "shutdown" },
                "actions": [{ "type": "respond", "body": null }]
            }
        ]
    });

    let config = WorkspaceConfig {
        packages: vec![PackageConfig {
            name: "pkg".to_string(),
            scenario,
            has_venv: true,
        }],
    };

    let (temp_dir, root) = support::setup_test_workspace(&config);
    let mut proxy = ProxyUnderTest::spawn(temp_dir, root.clone(), &root.join("pkg"));

    // Initialize
    let root_uri = support::path_to_uri(&root.join("pkg"));
    let init_resp = proxy.initialize(&root_uri).await;
    assert!(
        init_resp.result.is_some(),
        "initialize should return a result"
    );
    let caps = &init_resp.result.as_ref().unwrap()["capabilities"];
    assert!(
        caps.get("hoverProvider").is_some(),
        "capabilities should include hoverProvider"
    );

    // Initialized
    proxy.send_initialized().await;

    // Shutdown
    let shutdown_resp = proxy.shutdown_and_exit().await;
    // serde deserializes `"result": null` into `None` for Option<Value>,
    // so we check that the response is not an error rather than matching the exact value.
    assert!(
        shutdown_resp.error.is_none(),
        "shutdown response should not be an error"
    );
}
