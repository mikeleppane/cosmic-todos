use leptos::prelude::*;

#[server(HeartbeatServer, "/api")]
pub async fn heartbeat_server() -> Result<String, ServerFnError> {
    use chrono::Utc;

    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    Ok(format!("Server is running at {}", timestamp))
}
