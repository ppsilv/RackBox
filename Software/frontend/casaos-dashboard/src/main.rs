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
use sysinfo::{System, SystemExt};
use sysinfo::NetworkExt;

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

// Adicione esta estrutura junto com as outras
#[derive(Default)]
pub struct NetworkData {
    bytes_sent: u64,
    bytes_recv: u64,
    interfaces: Vec<(String, String)>,
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
    
    // Dados simulados de rede
 //   let network_data = NetworkData {
 //       bytes_sent: 14_567_890,
 //       bytes_recv: 23_456_789,
 //       interfaces: vec![
 //           ("eth0".to_string(), "UP".to_string()),
 //           ("wlan0".to_string(), "DOWN".to_string()),
 //       ],
 //   };
    // Coleta de dados de rede REAL
// Modifique a seção de rede para:
let network_data = {
    let mut sys = System::new();

    sys.refresh_all();  // Atualiza todos os dados do sistema
    
    let mut data = NetworkData::default();
    
    // Para cada interface de rede
    for (interface_name, network_data) in sys.networks() {
        data.bytes_sent += network_data.total_transmitted();
        data.bytes_recv += network_data.received();
        data.interfaces.push((
            interface_name.to_string(),
            if network_data.received() > 0 || network_data.total_transmitted() > 0 {
                "UP".to_string()
            } else {
                "DOWN".to_string()
            },
        ));
    }
    
    data
};


    let mut sys = System::new_all();
    sys.refresh_networks();

    let mut network_data = NetworkData::default();
    for (interface_name, network) in sys.networks() {
        network_data.bytes_sent += network.total_transmitted();
        network_data.bytes_recv += network.total_received();
        network_data.interfaces.push((
            interface_name.clone(),
            if network.total_received() > 0 || network.total_transmitted() > 0 {
                "UP".to_string()
            } else {
                "DOWN".to_string()
            },
        ));
    }

    let html = format!(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset="UTF-8">
        <title>CasaOS Dashboard</title>
        <style>
            :root {{
                --primary-color: #2A3950;
                --header-bg: #2A3950;
                --accent-color: #00C2FF;
                --success-color: #00D1A9;
                --danger-color: #FF4757;
                --text-light: #FFFFFF;
            }}

            body {{
                background: #f5f6fa;
                margin: 0;
                font-family: 'Segoe UI', system-ui, sans-serif;
            }}

            .container {{
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
                gap: 20px;
                max-width: 1400px;
                margin: 20px auto;
                padding: 0 20px;
            }}

            .panel {{
                background: white;
                border-radius: 12px;
                box-shadow: 0 4px 12px rgba(0,0,0,0.08);
                padding: 20px;
                min-height: 300px;
            }}

            .clients-box {{
                max-height: 70vh;
                overflow-y: auto;
                padding: 15px;
            }}

            .client-list {{
                display: grid;
                gap: 8px;
            }}

            .client-item-header {{
                display: grid;
                grid-template-columns: 80px 1fr 100px;
                align-items: center;
                gap: 15px;
                padding: 12px 15px;
                background: var(--header-bg);
                color: var(--text-light);
                border-radius: 6px;
                position: sticky;
                top: 0;
                z-index: 1;
            }}

            .client-item {{
                display: grid;
                grid-template-columns: 80px 1fr 100px;
                align-items: center;
                gap: 15px;
                padding: 12px 15px;
                background: #f8f9ff;
                border-radius: 6px;
                transition: all 0.2s ease;
                cursor: pointer;
            }}

            .client-item:hover {{
                transform: translateY(-2px);
                box-shadow: 0 3px 6px rgba(0,0,0,0.08);
                background: white;
            }}

            .ip-status-group {{
                display: flex;
                align-items: center;
                gap: 10px;
            }}

            .ip-cell {{
                white-space: nowrap;
                overflow: hidden;
                text-overflow: ellipsis;
            }}

            .status-cell {{
                padding: 4px 8px;
                border-radius: 12px;
                font-weight: 500;
                font-size: 0.9em;
                width: 80px;
                text-align: center;
            }}

            .status-active {{ background: rgba(0, 209, 169, 0.15); color: var(--success-color); }}
            .status-inactive {{ background: rgba(255, 71, 87, 0.15); color: var(--danger-color); }}

            .analytics-box {{
                padding: 20px;
                background: var(--header-bg);
                color: var(--text-light);
                border-radius: 8px;
            }}

            .network-panel {{
                display: grid;
                grid-template-columns: 1fr 1fr;
                gap: 20px;
            }}

            .interface-item {{
                display: flex;
                justify-content: space-between;
                align-items: center;
                padding: 8px;
                background: white;
                border-radius: 6px;
                box-shadow: 0 2px 4px rgba(0,0,0,0.05);
            }}

            .interface-status {{
                width: 10px;
                height: 10px;
                border-radius: 50%;
                margin-left: 10px;
            }}

            .status-up {{ background: var(--success-color); }}
            .status-down {{ background: var(--danger-color); }}
        </style>
    </head>
    <body>
        <div class="container">
            <!-- Painel de Máquinas -->
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
                        </div>
                        {}
                    </div>
                </div>
            </div>

            <!-- Painel de Analytics -->
            <div class="panel">
                <div class="analytics-box">
                    <h2>📊 Analytics</h2>
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

            <!-- Painel de Rede -->
            <div class="panel network-panel">
                <div class="network-metric">
                    <h3>🌐 Network Traffic</h3>
                    <div class="network-stats">
                        <div class="stat-item">
                            <span>Sent:</span>
                            <span class="stat-value">{}</span>
                        </div>
                        <div class="stat-item">
                            <span>Received:</span>
                            <span class="stat-value">{}</span>
                        </div>
                    </div>
                </div>

                <div class="network-metric">
                    <h3>🔌 Interfaces</h3>
                    <div class="interface-list">
                        {}
                    </div>
                </div>
            </div>
        </div>
    </body>
    </html>
    "#,
    clients_html(&clients),
    clients.len(),
    clients.values().filter(|c| c.status.eq_ignore_ascii_case("active")).count(),
    clients.values().filter(|c| c.status.eq_ignore_ascii_case("inactive")).count(),
    format_bytes(network_data.bytes_sent),
    format_bytes(network_data.bytes_recv),
    interfaces_html(network_data.interfaces)
    );

    HttpResponse::Ok().content_type("text/html").body(html)
}



fn clients_html(clients: &HashMap<u32, Client>) -> String {
    clients.values().map(|client| {
        let status_class = match client.status.to_lowercase().as_str() {
            "active" => "status-active",
            "inactive" => "status-inactive",
            _ => "status-pending"
        };
        
        format!(r#"
            <div class="client-item">
                <div>#{}</div>
                <div class="ip-status-group">
                    <div class="ip-cell">{}</div>
                    <div class="status-cell {}">{}</div>
                </div>
                <div>{}</div>
            </div>"#,
            client.id,
            client.ip,
            status_class,
            client.status,
            client.port
        )
    }).collect()
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_000_000 {
        format!("{:.2} MB", bytes as f64 / 1_000_000.0)
    } else {
        format!("{:.2} KB", bytes as f64 / 1_000.0)
    }
}

fn interfaces_html(interfaces: Vec<(String, String)>) -> String {
    interfaces.iter().map(|(name, status)| {
        format!(r#"
            <div class="interface-item">
                <span>{}</span>
                <div style="display: flex; align-items: center;">
                    <span class="data-unit">{}</span>
                    <div class="interface-status {}"></div>
                </div>
            </div>"#,
            name,
            status,
            if status == "UP" { "status-up" } else { "status-down" }
        )
    }).collect()
}
