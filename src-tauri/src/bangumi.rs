//! Bangumi (bgm.tv) API 查询模块
//!
//! 搜索使用旧版 GET API (`/search/subject/{keyword}`)，因为 v0 API 过滤了 NSFW 游戏。
//! 详情获取使用双轨策略：优先 v0 API（有标签+平台），失败时回退旧 API。
//!
//! 调试追踪：所有 API 请求/响应自动写入 %APPDATA%/KurisuGal/Logs/bangumi_trace.jsonl

use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Write;
use std::sync::Mutex;

const BGM_API_BASE: &str = "https://api.bgm.tv";
const USER_AGENT: &str =
    "KurisuGal/1.3.0 (https://github.com/2832886442/KurisuGalManager; galgame-manager)";
const REQUEST_TIMEOUT: u64 = 15;
const CONNECT_TIMEOUT: u64 = 8;

// ======================== 返回给前端的数据结构 ========================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiSearchItem {
    pub id: u32,
    pub name: String,
    pub name_cn: String,
    pub summary: String,
    pub image: String,
    pub image_large: String,
    pub date: String,
    pub score: f32,
    pub rank: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiGameDetail {
    pub id: u32,
    pub name: String,
    pub name_cn: String,
    pub summary: String,
    pub image: String,
    pub image_large: String,
    pub date: String,
    pub score: f32,
    pub rank: u32,
    pub tags: Vec<String>,
    pub platform: String,
}

// ======================== 调试追踪 ========================

static TRACE_FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);

fn get_trace_file() -> &'static Mutex<Option<std::fs::File>> {
    let mut guard = TRACE_FILE.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_none() {
        let trace_path = crate::path_manager::bangumi_trace_file();
        if let Some(parent) = trace_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&trace_path)
        {
            *guard = Some(f);
        }
    }
    drop(guard);
    &TRACE_FILE
}

fn trace_write(entry: serde_json::Value) {
    let line = serde_json::to_string(&entry).unwrap_or_default();
    let mut guard = get_trace_file().lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref mut f) = *guard {
        let _ = writeln!(f, "{}", line);
        let _ = f.flush();
    }
}

fn trace_now() -> String {
    chrono::Local::now().format("%H:%M:%S%.3f").to_string()
}

// ======================== 辅助函数 ========================

fn mk_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .default_headers({
            let mut h = reqwest::header::HeaderMap::new();
            h.insert(reqwest::header::ACCEPT, "application/json".parse().unwrap());
            h
        })
        .connect_timeout(std::time::Duration::from_secs(CONNECT_TIMEOUT))
        .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT))
        .build()
        .unwrap_or_default()
}

/// 检测 API 错误响应格式（兼容 v0: {"code": N, "error": "..."} 和旧API: {"code": 404, "error": "..."}）
fn check_api_error(json: &Value) -> Option<String> {
    if let Some(code) = json.get("code").and_then(|v| v.as_u64()) {
        if code != 0 {
            let msg = json
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("未知错误");
            warn!("Bangumi API 错误: code={}, error={}", code, msg);
            return Some(format!("Bangumi API 返回错误 (code={}): {}", code, msg));
        }
    }
    None
}

fn get_str(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string()
}

fn get_f32_obj(obj: &Value, key: &str) -> f32 {
    if !obj.is_object() {
        return 0.0;
    }
    obj.get(key)
        .and_then(|x| x.as_f64())
        .map(|n| n as f32)
        .unwrap_or(0.0)
}

fn get_u32(v: &Value, key: &str) -> u32 {
    v.get(key)
        .and_then(|x| x.as_u64())
        .map(|n| n as u32)
        .unwrap_or(0)
}

fn get_u32_obj(obj: &Value, key: &str) -> u32 {
    if !obj.is_object() {
        return 0;
    }
    obj.get(key)
        .and_then(|x| x.as_u64())
        .map(|n| n as u32)
        .unwrap_or(0)
}

fn get_img(v: &Value, key: &str, size: &str) -> String {
    v.get(key)
        .and_then(|x| x.get(size))
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string()
}

/// 发送 GET 请求并解析 JSON
async fn fetch_json(url: &str) -> Result<Value, String> {
    let client = mk_client();
    let start = std::time::Instant::now();

    let resp = client.get(url).send().await.map_err(|e| {
        let err_msg = if e.is_timeout() {
            "请求超时，请检查网络连接".to_string()
        } else if e.is_connect() {
            "无法连接到 Bangumi API，请检查网络".to_string()
        } else {
            format!("请求失败: {}", e)
        };
        trace_write(serde_json::json!({
            "time": trace_now(), "type": "GET_ERROR", "url": url,
            "error": err_msg, "elapsed_ms": start.elapsed().as_millis()
        }));
        err_msg
    })?;

    let status = resp.status();
    let elapsed_ms = start.elapsed().as_millis();
    let text = resp
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    // 仅在前200字符时记录全量，避免trace文件过大
    if text.len() <= 200 {
        trace_write(serde_json::json!({
            "time": trace_now(), "type": "GET_RESPONSE", "url": url,
            "status": status.as_u16(), "elapsed_ms": elapsed_ms, "body": text
        }));
    } else {
        let preview: String = text.chars().take(200).collect();
        trace_write(serde_json::json!({
            "time": trace_now(), "type": "GET_RESPONSE", "url": url,
            "status": status.as_u16(), "elapsed_ms": elapsed_ms,
            "body_len": text.len(), "body_preview": preview
        }));
    }

    if !status.is_success() {
        let preview = text.chars().take(200).collect::<String>();
        warn!("Bangumi API 返回 {}: {}", status, preview);
        return Err(format!("API 返回错误 ({}): {}", status.as_u16(), preview));
    }

    debug!(
        "Bangumi 响应 (前500字符): {}",
        text.chars().take(500).collect::<String>()
    );

    serde_json::from_str::<Value>(&text).map_err(|e| {
        let preview200: String = text.chars().take(200).collect();
        let err = format!("JSON 解析失败: {} (响应前200字符: {})", e, preview200);
        let preview500: String = text.chars().take(500).collect();
        trace_write(serde_json::json!({
            "time": trace_now(), "type": "GET_PARSE_ERROR", "url": url,
            "error": err, "body_preview": preview500
        }));
        err
    })
}

// ======================== 解析函数 ========================

/// 解析旧 API 搜索/详情响应中的条目（兼容旧 API 和 v0 API）
/// 旧 API 用 `air_date`，v0 用 `date`；rank 旧 API 在顶层，v0 在 rating 下
fn parse_subject_item(item: &Value) -> BangumiSearchItem {
    let rating = item.get("rating").unwrap_or(&Value::Null);
    // 旧 API 使用 air_date，v0 使用 date；都尝试
    let date = get_str(item, "air_date");
    let date = if date.is_empty() {
        get_str(item, "date")
    } else {
        date
    };
    // 旧 API 的 rank 在顶层（i32），v0 的 rank 在 rating 下
    let rank = {
        let r = get_u32(item, "rank");
        if r > 0 {
            r
        } else {
            get_u32_obj(rating, "rank")
        }
    };

    BangumiSearchItem {
        id: get_u32(item, "id"),
        name: get_str(item, "name"),
        name_cn: get_str(item, "name_cn"),
        summary: get_str(item, "summary"),
        image: get_img(item, "images", "medium"),
        image_large: get_img(item, "images", "large"),
        date,
        score: get_f32_obj(rating, "score"),
        rank,
    }
}

/// 解析为详情（与 search item 类似，但额外有 tags 和 platform）
fn parse_detail_item(item: &Value) -> BangumiGameDetail {
    let base = parse_subject_item(item);
    // tags: v0 API 有 tags 数组，旧 API 没有
    let tags: Vec<String> = item
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| {
                    let count = t.get("count").and_then(|c| c.as_u64()).unwrap_or(0);
                    if count >= 3 {
                        t.get("name")
                            .and_then(|n| n.as_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .take(10)
                .collect()
        })
        .unwrap_or_default();
    let platform = get_str(item, "platform");

    BangumiGameDetail {
        id: base.id,
        name: base.name,
        name_cn: base.name_cn,
        summary: base.summary,
        image: base.image,
        image_large: base.image_large,
        date: base.date,
        score: base.score,
        rank: base.rank,
        tags,
        platform,
    }
}

// ======================== API 函数 ========================

/// 搜索 Bangumi 游戏
/// 使用旧版 GET API: `/search/subject/{keyword}?type=4&responseGroup=large&max_results=25`
/// 旧版 API 不会过滤 NSFW 游戏（v0 API 会）
pub async fn search_games(keyword: &str) -> Result<Vec<BangumiSearchItem>, String> {
    let encoded = urlencoding::encode(keyword);
    let url = format!(
        "{}/search/subject/{}?type=4&responseGroup=large&max_results=25",
        BGM_API_BASE, encoded
    );
    info!("Bangumi 搜索: {} (旧版 API)", keyword);

    let json = fetch_json(&url).await?;

    // 检测 API 错误
    if let Some(err) = check_api_error(&json) {
        trace_write(serde_json::json!({
            "time": trace_now(), "type": "SEARCH_API_ERROR",
            "keyword": keyword, "error": err
        }));
        return Err(err);
    }

    // 旧 API 响应：{ "results": N, "list": [...] }
    let list = match json.get("list") {
        Some(Value::Array(arr)) => arr,
        Some(Value::Null) | None => {
            info!("搜索无结果: {}", keyword);
            trace_write(serde_json::json!({
                "time": trace_now(), "type": "SEARCH_NO_RESULTS", "keyword": keyword
            }));
            return Ok(vec![]);
        }
        other => {
            warn!("搜索响应 list 字段格式异常: {:?}", other);
            return Ok(vec![]);
        }
    };

    let items: Vec<BangumiSearchItem> = list
        .iter()
        .filter(|item| get_u32(item, "type") == 4) // 只保留游戏类型
        .map(parse_subject_item)
        .collect();

    info!("Bangumi 搜索「{}」返回 {} 条结果", keyword, items.len());

    trace_write(serde_json::json!({
        "time": trace_now(), "type": "SEARCH_RESULT", "keyword": keyword,
        "count": items.len(),
        "items": items.iter().map(|i| serde_json::json!({
            "id": i.id, "name": i.name, "name_cn": i.name_cn,
            "has_summary": !i.summary.is_empty(), "has_image": !i.image.is_empty(),
            "date": i.date, "score": i.score, "rank": i.rank
        })).collect::<Vec<_>>()
    }));

    Ok(items)
}

/// 获取单个游戏详情（双轨策略）
/// 1. 优先尝试 v0 API（GET /v0/subjects/{id}），可获得标签和平台信息
/// 2. 如果 v0 API 返回 404（NSFW 游戏），回退到旧版 API（GET /subject/{id}?responseGroup=large）
pub async fn get_game_detail(subject_id: u32) -> Result<BangumiGameDetail, String> {
    // === 策略 1: 尝试 v0 API ===
    let v0_url = format!("{}/v0/subjects/{}", BGM_API_BASE, subject_id);
    info!("获取详情 (v0): id={}", subject_id);

    match fetch_json(&v0_url).await {
        Ok(json) if check_api_error(&json).is_none() => {
            let name = get_str(&json, "name");
            let name_cn = get_str(&json, "name_cn");
            let summary = get_str(&json, "summary");
            if !name.is_empty() || !name_cn.is_empty() || !summary.is_empty() {
                let detail = parse_detail_item(&json);
                trace_write(serde_json::json!({
                    "time": trace_now(), "type": "DETAIL_RESULT", "api": "v0",
                    "subject_id": subject_id, "name": detail.name, "name_cn": detail.name_cn,
                    "has_summary": !detail.summary.is_empty(), "tag_count": detail.tags.len()
                }));
                return Ok(detail);
            }
            warn!("v0 API 返回数据无有效字段 (id={})，回退旧API", subject_id);
        }
        Ok(json) => {
            // v0 API 返回了错误（如 404 code）
            let err = check_api_error(&json).unwrap_or_else(|| "未知v0错误".into());
            warn!("v0 API 错误 (id={}): {}，回退旧API", subject_id, err);
        }
        Err(e) => {
            warn!("v0 API 请求失败 (id={}): {}，回退旧API", subject_id, e);
        }
    }

    // === 策略 2: 回退到旧版 API ===
    let old_url = format!(
        "{}/subject/{}?responseGroup=large",
        BGM_API_BASE, subject_id
    );
    info!("获取详情 (旧API): id={}", subject_id);

    let json = fetch_json(&old_url).await?;

    if let Some(err) = check_api_error(&json) {
        trace_write(serde_json::json!({
            "time": trace_now(), "type": "DETAIL_API_ERROR", "api": "old",
            "subject_id": subject_id, "error": err
        }));
        return Err(err);
    }

    let name = get_str(&json, "name");
    let name_cn = get_str(&json, "name_cn");
    let summary = get_str(&json, "summary");
    if name.is_empty() && name_cn.is_empty() && summary.is_empty() {
        let keys: Vec<&str> = json
            .as_object()
            .map(|o| o.keys().map(|k| k.as_str()).collect())
            .unwrap_or_default();
        warn!("旧API 详情缺少字段 (id={}): {:?}", subject_id, keys);
        return Err(format!(
            "API 返回数据异常：缺少 name/name_cn/summary 字段 (id={})",
            subject_id
        ));
    }

    let detail = parse_detail_item(&json);
    trace_write(serde_json::json!({
        "time": trace_now(), "type": "DETAIL_RESULT", "api": "old",
        "subject_id": subject_id, "name": detail.name, "name_cn": detail.name_cn,
        "has_summary": !detail.summary.is_empty(), "tag_count": detail.tags.len()
    }));

    Ok(detail)
}

// ======================== 封面下载 ========================

/// 下载封面图片并转为 Base64 data URI
pub async fn download_cover_as_base64(image_url: &str) -> Result<Vec<u8>, String> {
    if image_url.is_empty() {
        return Err("图片 URL 为空".into());
    }

    let url = if image_url.starts_with("http://") {
        image_url.replacen("http://", "https://", 1)
    } else {
        image_url.to_string()
    };

    info!("下载封面: {}", url);
    let client = mk_client();
    let start = std::time::Instant::now();

    let resp = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(20))
        .send()
        .await
        .map_err(|e| {
            let err = if e.is_timeout() {
                "封面下载超时".into()
            } else {
                format!("封面下载失败: {}", e)
            };
            trace_write(serde_json::json!({
                "time": trace_now(), "type": "COVER_DOWNLOAD_ERROR",
                "url": url, "error": err, "elapsed_ms": start.elapsed().as_millis()
            }));
            err
        })?;

    if !resp.status().is_success() {
        let err = format!("封面下载 HTTP {}", resp.status());
        trace_write(serde_json::json!({
            "time": trace_now(), "type": "COVER_HTTP_ERROR",
            "url": url, "status": resp.status().as_u16()
        }));
        return Err(err);
    }

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("读取封面数据失败: {}", e))?
        .to_vec();

    if bytes.len() > 2 * 1024 * 1024 {
        warn!("封面图片过大 ({} bytes)", bytes.len());
    }

    trace_write(serde_json::json!({
        "time": trace_now(), "type": "COVER_DOWNLOAD_OK",
        "url": url, "size_bytes": bytes.len(),
        "elapsed_ms": start.elapsed().as_millis()
    }));
    debug!("封面下载完成: {} bytes", bytes.len());
    Ok(bytes)
}

pub fn detect_image_mime(data: &[u8]) -> &'static str {
    if data.len() < 4 {
        return "image/jpeg";
    }
    match &data[0..4] {
        [0x89, 0x50, 0x4E, 0x47] => "image/png",
        [0x47, 0x49, 0x46, 0x38] => "image/gif",
        [0x52, 0x49, 0x46, 0x46] => "image/webp",
        [0xFF, 0xD8, 0xFF, _] => "image/jpeg",
        _ => "image/jpeg",
    }
}

/// 将图片字节转为 Base64 data URI
pub fn bytes_to_data_uri(data: &[u8]) -> String {
    let mime = detect_image_mime(data);
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data);
    format!("data:{};base64,{}", mime, b64)
}
