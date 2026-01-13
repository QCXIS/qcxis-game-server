use crate::metrics::{MetricsCollector, ServerMetrics};
use crate::state::AppState;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::{error, info};

pub async fn start_http_server(
    addr: SocketAddr,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    let collector = Arc::new(Mutex::new(MetricsCollector::new()));
    let listener = TcpListener::bind(addr).await?;

    info!("📊 HTTP Status server listening on: http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let state = state.clone();
        let collector = collector.clone();

        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| {
                        handle_request(req, state.clone(), collector.clone())
                    }),
                )
                .await
            {
                error!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
    state: AppState,
    collector: Arc<Mutex<MetricsCollector>>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let path = req.uri().path();

    match path {
        "/status" => {
            let mut collector = collector.lock().await;
            let metrics = collector.collect(&state);
            let json = serde_json::to_string_pretty(&metrics).unwrap();

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(Full::new(Bytes::from(json)))
                .unwrap())
        }
        "/health" => {
            let response = serde_json::json!({
                "status": "healthy",
                "service": "qcxis-game-server"
            });

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(Full::new(Bytes::from(response.to_string())))
                .unwrap())
        }
        "/metrics" => {
            // Prometheus-compatible plain text format
            let mut collector = collector.lock().await;
            let metrics: ServerMetrics = collector.collect(&state);

            let prometheus_metrics = format!(
                "# HELP qcxis_cpu_usage_percent CPU usage percentage\n\
                 # TYPE qcxis_cpu_usage_percent gauge\n\
                 qcxis_cpu_usage_percent {}\n\
                 # HELP qcxis_memory_used_mb Memory used in MB\n\
                 # TYPE qcxis_memory_used_mb gauge\n\
                 qcxis_memory_used_mb {}\n\
                 # HELP qcxis_memory_total_mb Total memory in MB\n\
                 # TYPE qcxis_memory_total_mb gauge\n\
                 qcxis_memory_total_mb {}\n\
                 # HELP qcxis_memory_used_percent Memory usage percentage\n\
                 # TYPE qcxis_memory_used_percent gauge\n\
                 qcxis_memory_used_percent {}\n\
                 # HELP qcxis_process_memory_mb Process memory in MB\n\
                 # TYPE qcxis_process_memory_mb gauge\n\
                 qcxis_process_memory_mb {}\n\
                 # HELP qcxis_total_games Total number of games\n\
                 # TYPE qcxis_total_games gauge\n\
                 qcxis_total_games {}\n\
                 # HELP qcxis_active_connections Active connections\n\
                 # TYPE qcxis_active_connections gauge\n\
                 qcxis_active_connections {}\n\
                 # HELP qcxis_total_players Total players connected\n\
                 # TYPE qcxis_total_players gauge\n\
                 qcxis_total_players {}\n\
                 # HELP qcxis_uptime_seconds Server uptime in seconds\n\
                 # TYPE qcxis_uptime_seconds counter\n\
                 qcxis_uptime_seconds {}\n",
                metrics.system.cpu_usage_percent,
                metrics.system.memory_used_mb,
                metrics.system.memory_total_mb,
                metrics.system.memory_used_percent,
                metrics.system.process_memory_mb,
                metrics.games.total_games,
                metrics.games.active_connections,
                metrics.games.total_players_connected,
                metrics.uptime_seconds
            );

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain; version=0.0.4")
                .header("Access-Control-Allow-Origin", "*")
                .body(Full::new(Bytes::from(prometheus_metrics)))
                .unwrap())
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(
                r#"{"error":"Not Found"}"#.to_string(),
            )))
            .unwrap()),
    }
}
