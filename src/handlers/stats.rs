use actix_web::{web, HttpResponse};
use sqlx::PgPool;

use crate::models::{ApiError, ApiResponse, DashboardStats};

pub async fn get_stats(pool: web::Data<PgPool>) -> HttpResponse {
    let stats = async {
        let total_products = sqlx::query_scalar!("SELECT COUNT(*) as \"count!\" FROM products")
            .fetch_one(pool.get_ref())
            .await?;

        let published_products = sqlx::query_scalar!(
            "SELECT COUNT(*) as \"count!\" FROM products WHERE is_published = true"
        )
        .fetch_one(pool.get_ref())
        .await?;

        let total_gallery_items =
            sqlx::query_scalar!("SELECT COUNT(*) as \"count!\" FROM gallery_items")
                .fetch_one(pool.get_ref())
                .await?;

        let total_employees = sqlx::query_scalar!("SELECT COUNT(*) as \"count!\" FROM users")
            .fetch_one(pool.get_ref())
            .await?;

        let active_employees =
            sqlx::query_scalar!("SELECT COUNT(*) as \"count!\" FROM users WHERE is_active = true")
                .fetch_one(pool.get_ref())
                .await?;

        Ok::<DashboardStats, sqlx::Error>(DashboardStats {
            total_products,
            published_products,
            total_gallery_items,
            total_employees,
            active_employees,
        })
    }
    .await;

    match stats {
        Ok(stats) => HttpResponse::Ok().json(ApiResponse::success(stats)),
        Err(e) => {
            log::error!("Error fetching stats: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to fetch stats"))
        }
    }
}