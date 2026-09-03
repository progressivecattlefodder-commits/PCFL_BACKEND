use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use qr_code::QrCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use user_agent_parser::UserAgentParser;
use uuid::Uuid;

#[derive(Serialize, sqlx::FromRow)]
pub struct DynamicQr {
    pub id: Uuid,
    pub slug: String,
    pub target_url: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct QrScanLog {
    pub id: Uuid,
    pub ip_address: Option<String>,
    pub device_type: Option<String>,
    pub os: Option<String>,
    pub browser: Option<String>,
    pub scanned_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct QrAnalyticsResponse {
    pub qr_info: DynamicQr,
    pub qr_svg: String,
    pub total_scans: i64,
    pub recent_logs: Vec<QrScanLog>,
}

#[derive(Deserialize)]
pub struct UpdateTargetPayload {
    pub new_target_url: String,
}

// 1. SCAN & REDIRECT ENDPOINT (Public)
pub async fn handle_qr_scan(
    path: web::Path<String>,
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let slug = path.into_inner();

    let qr = match sqlx::query_as::<_, DynamicQr>(
        "SELECT id, slug, target_url, updated_at FROM dynamic_qrs WHERE slug = $1",
    )
    .bind(&slug)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(record)) => record,
        _ => return HttpResponse::NotFound().body("QR Code not found"),
    };

    // Extract headers for device analytics
    let ip = req
        .connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string());
    
    let ua_string = req
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    // Minimal UA parsing
    let (device_type, os, browser) = if ua_string.contains("Mobile") || ua_string.contains("Android") || ua_string.contains("iPhone") {
        ("Mobile", "Mobile OS", "Mobile Browser")
    } else {
        ("Desktop", "Desktop OS", "Web Browser")
    };

    // Asynchronously log scan
    let pool_clone = pool.clone();
    let qr_id = qr.id;
    tokio::spawn(async move {
        let _ = sqlx::query!(
            "INSERT INTO qr_scan_logs (qr_id, ip_address, user_agent, device_type, os, browser) VALUES ($1, $2, $3, $4, $5, $6)",
            qr_id, ip, ua_string, device_type, os, browser
        )
        .execute(pool_clone.get_ref())
        .await;
    });

    // 302 Dynamic Redirect
    HttpResponse::Found()
        .append_header(("Location", qr.target_url))
        .finish()
}

// 2. ADMIN API: Get QR Details, SVG render & Analytics
pub async fn get_qr_analytics(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> impl Responder {
    let slug = "main";
    let host = req.connection_info().host().to_string();
    let scan_url = format!("http://{}/r/{}", host, slug);

    let qr_info = match sqlx::query_as::<_, DynamicQr>(
        "SELECT id, slug, target_url, updated_at FROM dynamic_qrs WHERE slug = $1",
    )
    .bind(slug)
    .fetch_one(pool.get_ref())
    .await {
        Ok(data) => data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Render SVG QR Vector directly in Rust
    let code = QrCode::new(scan_url.as_bytes()).unwrap();
    let qr_svg = code.render::<char>()
        .quiet_zone(false)
        .module_dimensions(6, 6)
        .build();

    let logs = sqlx::query_as::<_, QrScanLog>(
        "SELECT id, ip_address, device_type, os, browser, scanned_at FROM qr_scan_logs WHERE qr_id = $1 ORDER BY scanned_at DESC LIMIT 50",
    )
    .bind(qr_info.id)
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    let total_scans: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM qr_scan_logs WHERE qr_id = $1")
        .bind(qr_info.id)
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);

    HttpResponse::Ok().json(QrAnalyticsResponse {
        qr_info,
        qr_svg,
        total_scans,
        recent_logs: logs,
    })
}

// 3. ADMIN API: Update Target URL
pub async fn update_qr_target(
    payload: web::Json<UpdateTargetPayload>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let result = sqlx::query!(
        "UPDATE dynamic_qrs SET target_url = $1, updated_at = NOW() WHERE slug = 'main'",
        payload.new_target_url
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "success": true })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

// Route config module
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/r/{slug}").route(web::get().to(handle_qr_scan)))
       .service(web::resource("/api/admin/qr").route(web::get().to(get_qr_analytics)))
       .service(web::resource("/api/admin/qr/update").route(web::patch().to(update_qr_target)));
}