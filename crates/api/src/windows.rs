use axum::extract::{Path, State};
use axum::response::Json;
use chrono::Utc;
use poi_core::types::{OrbitalWindow, WindowType};
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::events::AppEvent;
use crate::models::{AppState, WindowCreateRequest};

#[derive(Debug, Deserialize)]
pub struct WindowQueryParams {
    pub window_type: Option<String>,
}

pub async fn list_windows(State(state): State<AppState>) -> ApiResult<Json<Vec<OrbitalWindow>>> {
    let windows = state.windows.read().await;
    Ok(Json(windows.clone()))
}

pub async fn create_window(
    State(state): State<AppState>,
    Json(req): Json<WindowCreateRequest>,
) -> ApiResult<Json<OrbitalWindow>> {
    let window_type = match req.window_type.to_lowercase().as_str() {
        "standard" => WindowType::Standard,
        "extended" => WindowType::Extended,
        "emergency" => WindowType::Emergency,
        _ => {
            return Err(ApiError::Validation(format!(
                "Invalid window type: {}. Must be 'standard', 'extended', or 'emergency'",
                req.window_type
            )))
        }
    };

    let window = OrbitalWindow::new(req.start_time, req.end_time, window_type);

    let mut windows = state.windows.write().await;
    windows.push(window.clone());
    drop(windows);

    let _ = state.event_tx.send(AppEvent::WindowOpened(window.clone()));

    Ok(Json(window))
}

pub async fn get_active_windows(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<OrbitalWindow>>> {
    let windows = state.windows.read().await;
    let now = Utc::now();
    let active: Vec<OrbitalWindow> = windows
        .iter()
        .filter(|w| w.start_time <= now && now <= w.end_time)
        .cloned()
        .collect();
    Ok(Json(active))
}

pub async fn get_window(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<OrbitalWindow>> {
    let windows = state.windows.read().await;
    let window = windows
        .iter()
        .find(|w| w.id == id)
        .ok_or_else(|| ApiError::NotFound(format!("Window {} not found", id)))?
        .clone();
    Ok(Json(window))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use chrono::Duration;

    use crate::auth::AuthConfig;
    use crate::config::ApiConfig;
    use crate::models::AppStateInner;

    fn test_state() -> AppState {
        Arc::new(AppStateInner::new(ApiConfig::default(), AuthConfig::default()))
    }

    #[tokio::test]
    async fn test_list_windows_empty() {
        let state = test_state();
        let resp = list_windows(State(state)).await.unwrap();
        assert!(resp.0.is_empty());
    }

    #[tokio::test]
    async fn test_create_window() {
        let state = test_state();
        let req = WindowCreateRequest {
            start_time: Utc::now(),
            end_time: Utc::now() + Duration::hours(1),
            window_type: "standard".into(),
        };
        let resp = create_window(State(state.clone()), Json(req)).await.unwrap();
        assert_eq!(resp.0.window_type, WindowType::Standard);
        let windows = list_windows(State(state)).await.unwrap();
        assert_eq!(windows.0.len(), 1);
    }

    #[tokio::test]
    async fn test_create_window_invalid_type() {
        let state = test_state();
        let req = WindowCreateRequest {
            start_time: Utc::now(),
            end_time: Utc::now() + Duration::hours(1),
            window_type: "invalid".into(),
        };
        let result = create_window(State(state), Json(req)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_window_not_found() {
        let state = test_state();
        let id = Uuid::new_v4();
        let result = get_window(State(state), Path(id)).await;
        assert!(result.is_err());
    }
}
