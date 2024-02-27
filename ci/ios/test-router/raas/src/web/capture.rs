use axum::{
    body::Body,
    extract::{Json, Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
};
use uuid::Uuid;

#[derive(serde::Deserialize, Clone)]
pub struct NewCapture {
    pub label: Uuid,
}

pub async fn start(
    State(state): State<super::State>,
    Json(capture): Json<NewCapture>,
) -> impl IntoResponse {
    let label = capture.label;

    let result = async {
        let mut state = state.capture.lock().await;
        state.start(label).await?;
        log::info!("Started capture for label {label}");
        Ok(())
    }
    .await;

    respond_with_result(result, StatusCode::OK)
}

pub async fn stop(Path(label): Path<Uuid>, State(state): State<super::State>) -> impl IntoResponse {
    let result = async {
        let mut state = state.capture.lock().await;
        state.stop(label).await?;
        log::info!("Stopped capture for label {label}");
        Ok(())
    }
    .await;

    respond_with_result(result, StatusCode::OK)
}

pub async fn get(Path(label): Path<Uuid>, State(state): State<super::State>) -> impl IntoResponse {
    let state = state.capture.lock().await;

    let stream = match state.get(label).await {
        Ok(stream) => stream,
        Err(err) => {
            return (StatusCode::SERVICE_UNAVAILABLE, format!("{err}\n")).into_response();
        }
    };

    let body = Body::from_stream(stream);
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/pcap"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        header::HeaderValue::from_static("attachment; filename=\"dump.pcap\""),
    );

    (headers, body).into_response()
}

fn respond_with_result(result: anyhow::Result<()>, success_code: StatusCode) -> impl IntoResponse {
    match result {
        Ok(_) => (success_code, String::new()),
        Err(err) => (StatusCode::SERVICE_UNAVAILABLE, format!("{err}\n")),
    }
}
