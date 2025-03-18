/*
sudo docker buildx build --platform linux/amd64 -t casaos-dashboard . --load --no-cache

sudo docker run -d -p 8080:8080 --name dashboard casaos-dashboard

curl http://localhost:8080/health

Para verificar se o servidor está funcionando
# Exemplo com diferentes status
curl -X POST -H "Content-Type: application/json" -d '{"Comando":{"client":{"id":1001,"ip":"192.168.1.101","status":"inactive","port":15847,"cid":1001}}}' http://localhost:8080/data

curl -X POST -H "Content-Type: application/json" -d '{"Comando":{"client":{"id":1002,"ip":"192.168.1.102","status":"pending","port":15848,"cid":1002}}}' http://localhost:8080/data

*/

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Client {
    id: u32,
    ip: String,
    status: String,
    port: u16,
    cid: u32,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    uptime: f64,
    clients_connected: usize,
}

struct AppState {
    clients: Arc<Mutex<HashMap<u32, Client>>>,
    start_time: SystemTime,
}

async fn receive_command(
    data: web::Json<serde_json::Value>,
    state: web::Data<AppState>,
) -> impl Responder {
    if let Some(command) = data.get("Comando").and_then(|c| c.get("client")) {
        if let Ok(client) = serde_json::from_value::<Client>(command.clone()) {
            let mut clients = state.clients.lock().unwrap();
            clients.insert(client.id, client);
            return HttpResponse::Ok().body("Dados recebidos");
        }
    }
    HttpResponse::BadRequest().body("Formato inválido")
}

async fn dashboardold(state: web::Data<AppState>) -> impl Responder {
    let clients = state.clients.lock().unwrap();

    let mut html = String::from(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>CasaOS Client Dashboard</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 20px; }
                .client-card { border: 1px solid #ddd; padding: 15px; margin: 10px; border-radius: 5px; }
                .status-active { background-color: #d4edda; }
            </style>
        </head>
        <body>
            <h1>Clientes Conectados</h1>
    "#);

    for client in clients.values() {
        html.push_str(&format!(r#"
            <div class="client-card status-{}">
                <h2>Client ID: {}</h2>
                <p>IP: {}</p>
                <p>Porta: {}</p>
                <p>Status: {}</p>
                <p>CID: {}</p>
            </div>
        "#,
        client.status.to_lowercase(),
        client.id,
        client.ip,
        client.port,
        client.status,
        client.cid));
    }

    html.push_str("</body></html>");
    HttpResponse::Ok().content_type("text/html").body(html)
}

async fn health_check(state: web::Data<AppState>) -> impl Responder {
    let clients = state.clients.lock().unwrap();

    let uptime = SystemTime::now()
        .duration_since(state.start_time)
        .unwrap_or(Duration::from_secs(0))
        .as_secs_f64();

    HttpResponse::Ok().json(HealthResponse {
        status: "healthy".to_string(),
        uptime: (uptime * 100.0).round() / 100.0,
        clients_connected: clients.len(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let clients = Arc::new(Mutex::new(HashMap::new()));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                clients: Arc::clone(&clients),
                start_time: SystemTime::now(),
            }))
            .route("/", web::get().to(dashboard))
            .route("/data", web::post().to(receive_command))
            .route("/health", web::get().to(health_check))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

async fn dashboard(state: web::Data<AppState>) -> impl Responder {
    let clients = state.clients.lock().unwrap();
    
    let active_count = clients.values().filter(|c| c.status.eq_ignore_ascii_case("active")).count();
    let inactive_count = clients.values().filter(|c| c.status.eq_ignore_ascii_case("inactive")).count();

    let mut html = String::from(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset="UTF-8">
        <title>CasaOS Client Dashboard</title>
        <style>
            :root {
                --primary-color: #2A3950;
                --header-bg: #2A3950;
                --accent-color: #00C2FF;
                --success-color: #00D1A9;
                --danger-color: #FF4757;
                --text-light: #FFFFFF;
            }

            body {
                background: #f5f6fa;
                margin: 0;
                font-family: 'Segoe UI', system-ui, sans-serif;
            }

            .container {
                display: grid;
                grid-template-columns: 1fr 300px;
                gap: 15px;
                max-width: 1200px;
                margin: 15px auto;
                padding: 0 15px;
                min-height: 100vh;
            }

            .panel {
                background: white;
                border-radius: 8px;
                box-shadow: 0 2px 8px rgba(0,0,0,0.1);
            }

            .clients-box {
                max-height: 80vh;
                overflow-y: auto;
                padding: 15px;
            }

            .client-list {
                display: grid;
                gap: 8px;
            }

            .client-item-header {
                display: grid;
                grid-template-columns: 80px minmax(150px, 2fr) 120px 80px;
                align-items: center;
                gap: 15px;
                padding: 12px 15px;
                background: var(--header-bg);
                color: var(--text-light);
                border-radius: 6px;
                position: sticky;
                top: 0;
                z-index: 1;
            }

            .client-item {
                display: grid;
                grid-template-columns: 80px minmax(150px, 2fr) 120px 80px;
                align-items: center;
                gap: 15px;
                padding: 12px 15px;
                background: #f8f9ff;
                border-radius: 6px;
                border-left: 3px solid;
                transition: all 0.2s ease;
                cursor: pointer;
            }

            .client-item:hover {
                transform: translateY(-2px);
                box-shadow: 0 3px 6px rgba(0,0,0,0.08);
                background: white;
            }

            .ip-status-group {
                display: flex;
                align-items: center;
                gap: 20px;
            }

            .ip-cell {
                white-space: nowrap;
                overflow: hidden;
                text-overflow: ellipsis;
                flex: 2;
            }

            .status-cell {
                flex: 1;
                min-width: 100px;
                padding-left: 10px;
            }

            .status-active { border-color: var(--success-color); }
            .status-inactive { border-color: var(--danger-color); }
            .status-pending { border-color: var(--accent-color); }

            .analytics-box {
                padding: 20px;
                background: var(--header-bg);
                color: var(--text-light);
                border-radius: 8px;
                position: sticky;
                top: 15px;
            }

            .analytics-box h2 {
                margin: 0 0 15px 0;
                font-size: 1.3em;
                display: flex;
                align-items: center;
                gap: 10px;
            }

            .stat-item {
                margin: 12px 0;
                display: flex;
                justify-content: space-between;
                align-items: center;
            }

            .stat-value {
                font-weight: 500;
                font-size: 1.1em;
            }
        </style>
    </head>
    <body>
        <div class="container">
            <div class="panel">
                <div class="clients-box">
                    <div class="client-list">
                        <div class="client-item-header">
                            <div>ID</div>
                            <div class="ip-status-group">
                                <span>IP Address</span>
                                <span>Status</span>
                            </div>
                            <div>Port</div>
                        </div>"#);

    // Adiciona os clientes
    for client in clients.values() {
        let status_class = match client.status.to_lowercase().as_str() {
            "active" => "status-active",
            "inactive" => "status-inactive",
            _ => "status-pending"
        };

        html.push_str(&format!(
            r#"<div class="client-item {}">
                <div>#{}</div>
                <div class="ip-status-group">
                    <div class="ip-cell">{}</div>
                    <div class="status-cell" style="color: {}">{}</div>
                </div>
                <div>{}</div>
            </div>"#,
            status_class,
            client.id,
            client.ip,
            match status_class {
                "status-active" => "var(--success-color)",
                "status-inactive" => "var(--danger-color)",
                _ => "var(--accent-color)"
            },
            client.status,
            client.port
        ));
    }

    // Fecha a seção de clientes e adiciona analytics
    html.push_str(&format!(
        r#"</div>
                </div>
            </div>
            <div class="panel">
                <div class="analytics-box">
                    <h2>📊 System Analytics</h2>
                    <div class="stat-item">
                        <span>Total Machines:</span>
                        <span class="stat-value">{}</span>
                    </div>
                    <div class="stat-item">
                        <span>Active:</span>
                        <span class="stat-value" style="color: var(--success-color)">{}</span>
                    </div>
                    <div class="stat-item">
                        <span>Inactive:</span>
                        <span class="stat-value" style="color: var(--danger-color)">{}</span>
                    </div>
                </div>
            </div>
        </div>
    </body>
    </html>"#,
        clients.len(),
        active_count,
        inactive_count
    ));

    HttpResponse::Ok().content_type("text/html").body(html)
}