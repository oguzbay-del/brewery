#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

use std::time::Duration;

use e2e::Context;
use reqwest::{Method, Response, StatusCode};
use serde_json::{Value, json};

const EVENT_TIMEOUT: Duration = Duration::from_secs(15);

/// Reads the next named SSE event. Comments and non-event blocks are ignored.
async fn next_sse_event(
    response: &mut Response,
    buffer: &mut String,
) -> Option<(String, String)> {
    loop {
        while let Some((end, delimiter_len)) = buffer
            .find("\r\n\r\n")
            .map(|end| (end, 4))
            .or_else(|| buffer.find("\n\n").map(|end| (end, 2)))
        {
            let block = buffer[..end].to_owned();
            buffer.drain(..end + delimiter_len);

            let mut event = None;
            let mut data = String::new();
            for line in block.lines().map(|line| line.trim_end_matches('\r')) {
                if let Some(value) = line.strip_prefix("event:") {
                    event = Some(value.trim().to_owned());
                } else if let Some(value) = line.strip_prefix("data:") {
                    if !data.is_empty() {
                        data.push('\n');
                    }
                    data.push_str(value.trim_start());
                }
            }

            if let Some(event) = event {
                return Some((event, data));
            }
        }

        let chunk = response.chunk().await.ok()??;
        buffer.push_str(&String::from_utf8_lossy(&chunk));
    }
}

async fn assert_event(
    response: &mut Response,
    buffer: &mut String,
    expected_type: &str,
    recipe_id: &str,
) {
    let event = tokio::time::timeout(EVENT_TIMEOUT, async {
        loop {
            let Some((event_type, data)) = next_sse_event(response, buffer).await else {
                panic!("SSE stream ended before {expected_type}");
            };
            if event_type == expected_type {
                break (event_type, data);
            }
        }
    })
    .await
    .unwrap();

    assert_eq!(event.0, expected_type);
    let data: Value = serde_json::from_str(&event.1).unwrap();
    assert_eq!(data["type"], expected_type);
    assert_eq!(
        data["entity"]["id"].as_str(),
        Some(recipe_id),
        "event data must identify the edited recipe"
    );
}

async fn assert_no_recipe_event(response: &mut Response, buffer: &mut String, recipe_id: &str) {
    let result = tokio::time::timeout(EVENT_TIMEOUT, async {
        loop {
            let Some((event_type, data)) = next_sse_event(response, buffer).await else {
                panic!("user B SSE stream ended during the isolation window");
            };
            assert!(
                !data.contains(recipe_id),
                "user B received {event_type} for user A's recipe"
            );
        }
    })
    .await;
    assert!(result.is_err(), "user B received an event during the isolation window");
}

#[tokio::test]
async fn edit_session_events_are_delivered_over_sse_and_isolated() {
    let Some(ctx) = Context::from_env() else {
        eprintln!("skipping E2E: E2E_BASE_URL is not set");
        return;
    };

    let user_a = ctx.login().await.unwrap();
    let workspace_a = ctx
        .create_workspace(&user_a, "home", "E2E Edit Session A")
        .await
        .unwrap();
    let ws_a = workspace_a["id"].as_str().unwrap();
    let recipe_response = ctx
        .create_recipe(&user_a, ws_a, "E2E Edit Session Recipe")
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    assert_eq!(recipe_response.status(), StatusCode::CREATED);
    let recipe: Value = recipe_response.json().await.unwrap();
    let recipe_id = recipe["id"].as_str().unwrap();

    let user_b = ctx.login().await.unwrap();
    let workspace_b = ctx
        .create_workspace(&user_b, "home", "E2E Edit Session B")
        .await
        .unwrap();
    let ws_b = workspace_b["id"].as_str().unwrap();

    let mut stream_a = ctx
        .request(
            Method::GET,
            &format!("/v1/workspaces/{ws_a}/events"),
            Some(&user_a),
        )
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let mut stream_b = ctx
        .request(
            Method::GET,
            &format!("/v1/workspaces/{ws_b}/events"),
            Some(&user_b),
        )
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let mut buffer_a = String::new();
    let mut buffer_b = String::new();

    let started = ctx
        .post_json(
            &format!("/v1/workspaces/{ws_a}/recipes/{recipe_id}/edit-session"),
            Some(&user_a),
            &json!({}),
        )
        .await
        .unwrap();
    assert_eq!(started.status(), StatusCode::OK);
    assert_event(
        &mut stream_a,
        &mut buffer_a,
        "recipe.edit_session_started",
        recipe_id,
    )
    .await;
    assert_no_recipe_event(&mut stream_b, &mut buffer_b, recipe_id).await;

    let released = ctx
        .request(
            Method::DELETE,
            &format!("/v1/workspaces/{ws_a}/recipes/{recipe_id}/edit-session"),
            Some(&user_a),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(released.status(), StatusCode::NO_CONTENT);
    assert_event(
        &mut stream_a,
        &mut buffer_a,
        "recipe.edit_session_released",
        recipe_id,
    )
    .await;
    assert_no_recipe_event(&mut stream_b, &mut buffer_b, recipe_id).await;
}
