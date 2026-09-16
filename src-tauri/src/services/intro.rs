use std::time::{Duration, Instant};

use reqwest::{header, redirect::Policy, Client};
use serde::Serialize;
use serde_json::Value;
use std::sync::{Mutex, OnceLock};

const SUPPORTERS_URL: &str = "https://cs2as.600318.xyz/api/supporters";
const UPSTREAM_URL: &str = "https://api.github.com/repos/ed0ard/CS2-Bot-Improver";
const MAX_BODY_BYTES: usize = 128 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_millis(2_200);
const CACHE_TTL: Duration = Duration::from_secs(900);
static CACHE: OnceLock<Mutex<Option<(Instant, IntroPublicPayload)>>> = OnceLock::new();

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupporterAcknowledgement {
    id: String,
    nickname: Option<String>,
    message: Option<String>,
    amount_cents: Option<u64>,
    platform: Option<String>,
    unit: Option<String>,
    visible_amount: Option<f64>,
    exchange_rate_cny: Option<f64>,
    amount_scope: Option<String>,
    source_label: Option<String>,
    occurred_at: Option<String>,
    sort_order: u64,
    is_visible: bool,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpstreamProjectSummary {
    full_name: String,
    description: String,
    url: String,
    license: String,
    stars: Option<u64>,
    forks: Option<u64>,
    pushed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntroSources {
    supporters: &'static str,
    upstream: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntroDiagnostics {
    supporters: &'static str,
    upstream: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntroPublicPayload {
    supporters: Vec<SupporterAcknowledgement>,
    upstream: UpstreamProjectSummary,
    sources: IntroSources,
    diagnostics: IntroDiagnostics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FetchFailure {
    Timeout,
    Network,
    Http,
    Invalid,
}

impl FetchFailure {
    fn code(self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::Network => "network",
            Self::Http => "http",
            Self::Invalid => "invalid",
        }
    }
}

pub async fn get_intro_public_data() -> IntroPublicPayload {
    if let Some(cache) = CACHE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .ok()
        .and_then(|guard| {
            guard
                .as_ref()
                .and_then(|(at, value)| (at.elapsed() < CACHE_TTL).then(|| value.clone()))
        })
    {
        return cache.clone();
    }
    let client = match Client::builder()
        .connect_timeout(REQUEST_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .redirect(Policy::none())
        .https_only(true)
        .build()
    {
        Ok(client) => client,
        Err(error) => {
            log::warn!("INTRO_NATIVE_CLIENT_FAILED error={error}");
            return fallback_payload(FetchFailure::Network, FetchFailure::Network);
        }
    };

    let supporters_client = client.clone();
    let upstream_client = client;
    let supporters_task = tauri::async_runtime::spawn(async move {
        let started = Instant::now();
        let result = fetch_supporters(&supporters_client).await;
        log_result("supporters", &result, started.elapsed());
        result
    });
    let upstream_task = tauri::async_runtime::spawn(async move {
        let started = Instant::now();
        let result = fetch_upstream(&upstream_client).await;
        log_result("upstream", &result, started.elapsed());
        result
    });

    let supporters_result = supporters_task.await.unwrap_or(Err(FetchFailure::Network));
    let upstream_result = upstream_task.await.unwrap_or(Err(FetchFailure::Network));
    let supporters_diagnostic = supporters_result.as_ref().err().copied();
    let upstream_diagnostic = upstream_result.as_ref().err().copied();

    let payload = IntroPublicPayload {
        supporters: supporters_result.unwrap_or_default(),
        upstream: upstream_result.unwrap_or_else(|_| fallback_upstream()),
        sources: IntroSources {
            supporters: if supporters_diagnostic.is_none() {
                "network"
            } else {
                "fallback"
            },
            upstream: if upstream_diagnostic.is_none() {
                "network"
            } else {
                "fallback"
            },
        },
        diagnostics: IntroDiagnostics {
            supporters: supporters_diagnostic
                .map(FetchFailure::code)
                .unwrap_or("ok"),
            upstream: upstream_diagnostic.map(FetchFailure::code).unwrap_or("ok"),
        },
    };
    if let Ok(mut guard) = CACHE.get_or_init(|| Mutex::new(None)).lock() {
        *guard = Some((Instant::now(), payload.clone()));
    }
    payload
}

fn fallback_payload(supporters: FetchFailure, upstream: FetchFailure) -> IntroPublicPayload {
    IntroPublicPayload {
        supporters: Vec::new(),
        upstream: fallback_upstream(),
        sources: IntroSources {
            supporters: "fallback",
            upstream: "fallback",
        },
        diagnostics: IntroDiagnostics {
            supporters: supporters.code(),
            upstream: upstream.code(),
        },
    }
}

fn log_result<T>(endpoint: &str, result: &Result<T, FetchFailure>, elapsed: Duration) {
    match result {
        Ok(_) => log::info!(
            "INTRO_NATIVE_FETCH endpoint={endpoint} reason=ok elapsed_ms={}",
            elapsed.as_millis()
        ),
        Err(reason) => log::warn!(
            "INTRO_NATIVE_FETCH endpoint={endpoint} reason={} elapsed_ms={}",
            reason.code(),
            elapsed.as_millis()
        ),
    }
}

async fn fetch_json(client: &Client, url: &str, github: bool) -> Result<Value, FetchFailure> {
    let mut request = client.get(url).header(
        header::ACCEPT,
        if github {
            "application/vnd.github+json"
        } else {
            "application/json"
        },
    );
    if github {
        request = request.header(header::USER_AGENT, "CS2AS05/0.5.6");
    }
    let response = request.send().await.map_err(classify_request_error)?;
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_owned();
    if let Some(length) = response.content_length() {
        if length as usize > MAX_BODY_BYTES {
            return Err(FetchFailure::Invalid);
        }
    }
    let body = response.bytes().await.map_err(classify_request_error)?;
    parse_http_json(status, &content_type, &body)
}

fn classify_request_error(error: reqwest::Error) -> FetchFailure {
    if error.is_timeout() {
        FetchFailure::Timeout
    } else {
        FetchFailure::Network
    }
}

fn parse_http_json(status: u16, content_type: &str, body: &[u8]) -> Result<Value, FetchFailure> {
    if !(200..300).contains(&status) {
        return Err(FetchFailure::Http);
    }
    if !content_type
        .to_ascii_lowercase()
        .starts_with("application/json")
        || body.len() > MAX_BODY_BYTES
    {
        return Err(FetchFailure::Invalid);
    }
    serde_json::from_slice(body).map_err(|_| FetchFailure::Invalid)
}

async fn fetch_supporters(client: &Client) -> Result<Vec<SupporterAcknowledgement>, FetchFailure> {
    parse_supporters(fetch_json(client, SUPPORTERS_URL, false).await?)
}

fn parse_supporters(value: Value) -> Result<Vec<SupporterAcknowledgement>, FetchFailure> {
    let rows = value
        .get("supporters")
        .and_then(Value::as_array)
        .ok_or(FetchFailure::Invalid)?;
    Ok(rows
        .iter()
        .take(50)
        .filter_map(|row| {
            let id = clean_required(row.get("id")?.as_str()?, 80)?;
            let amount_cents = row.get("amountCents").and_then(Value::as_u64);
            let visible_amount = row.get("visibleAmount").and_then(Value::as_f64);
            if amount_cents.is_none() && visible_amount.is_none() {
                return None;
            }
            Some(SupporterAcknowledgement {
                id,
                nickname: clean_optional(row.get("nickname").and_then(Value::as_str), 80),
                message: clean_optional(row.get("message").and_then(Value::as_str), 180),
                amount_cents,
                platform: row
                    .get("platform")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                unit: row.get("unit").and_then(Value::as_str).map(str::to_owned),
                visible_amount,
                exchange_rate_cny: row.get("exchangeRateCny").and_then(Value::as_f64),
                amount_scope: row
                    .get("amountScope")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                source_label: clean_optional(row.get("sourceLabel").and_then(Value::as_str), 100),
                occurred_at: clean_optional(row.get("occurredAt").and_then(Value::as_str), 40),
                sort_order: row.get("sortOrder").and_then(Value::as_u64).unwrap_or(0),
                is_visible: row
                    .get("isVisible")
                    .and_then(Value::as_bool)
                    .unwrap_or(true),
                created_at: clean_optional(row.get("createdAt").and_then(Value::as_str), 40)
                    .unwrap_or_default(),
                updated_at: clean_optional(row.get("updatedAt").and_then(Value::as_str), 40)
                    .unwrap_or_default(),
            })
        })
        .collect())
}

async fn fetch_upstream(client: &Client) -> Result<UpstreamProjectSummary, FetchFailure> {
    parse_upstream(fetch_json(client, UPSTREAM_URL, true).await?)
}

fn parse_upstream(value: Value) -> Result<UpstreamProjectSummary, FetchFailure> {
    let full_name = clean_required(
        value.get("full_name").and_then(Value::as_str).unwrap_or(""),
        100,
    )
    .ok_or(FetchFailure::Invalid)?;
    let license = value
        .get("license")
        .and_then(|item| item.get("spdx_id"))
        .and_then(Value::as_str)
        .and_then(|item| clean_required(item, 40))
        .unwrap_or_else(|| "AGPL-3.0".to_owned());
    Ok(UpstreamProjectSummary {
        full_name,
        description: clean_optional(value.get("description").and_then(Value::as_str), 240)
            .unwrap_or_else(|| fallback_upstream().description),
        url: clean_required(
            value.get("html_url").and_then(Value::as_str).unwrap_or(""),
            300,
        )
        .unwrap_or_else(|| fallback_upstream().url),
        license,
        stars: value.get("stargazers_count").and_then(Value::as_u64),
        forks: value.get("forks_count").and_then(Value::as_u64),
        pushed_at: clean_optional(value.get("pushed_at").and_then(Value::as_str), 40),
    })
}

fn clean_required(value: &str, max: usize) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.chars().take(max).collect())
    }
}

fn clean_optional(value: Option<&str>, max: usize) -> Option<String> {
    value.and_then(|item| clean_required(item, max))
}

fn fallback_upstream() -> UpstreamProjectSummary {
    UpstreamProjectSummary {
        full_name: "ed0ard/CS2-Bot-Improver".to_owned(),
        description: "CS2 Bot 行为增强项目，本助手基于其能力构建。".to_owned(),
        url: "https://github.com/ed0ard/CS2-Bot-Improver".to_owned(),
        license: "AGPL-3.0".to_owned(),
        stars: None,
        forks: None,
        pushed_at: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_and_bounds_supporters() {
        let supporters = parse_supporters(json!({ "supporters": [
            { "id": "one", "nickname": "旅人", "message": "同行", "amountCents": 2000, "sortOrder": 1, "isVisible": true },
            { "id": "", "amountCents": 100 },
            { "id": "negative", "amountCents": -1 }
        ]})).unwrap();
        assert_eq!(supporters.len(), 1);
        assert_eq!(supporters[0].id, "one");
        assert_eq!(supporters[0].amount_cents, Some(2000));
    }

    #[test]
    fn classifies_http_and_invalid_responses() {
        assert_eq!(
            parse_http_json(503, "application/json", b"{}").unwrap_err(),
            FetchFailure::Http
        );
        assert_eq!(
            parse_http_json(200, "text/html", b"{}").unwrap_err(),
            FetchFailure::Invalid
        );
        assert_eq!(
            parse_http_json(200, "application/json", b"not-json").unwrap_err(),
            FetchFailure::Invalid
        );
    }

    #[test]
    fn supports_each_endpoint_failing_independently() {
        assert!(parse_supporters(json!({ "supporters": [] })).is_ok());
        assert_eq!(
            parse_upstream(json!({ "message": "rate limited" })).unwrap_err(),
            FetchFailure::Invalid
        );
        assert_eq!(fallback_upstream().full_name, "ed0ard/CS2-Bot-Improver");
    }
}
