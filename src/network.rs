use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::mpsc;
use std::net::TcpStream;
use tungstenite::{connect, Message, WebSocket, stream::MaybeTlsStream};

/// Commands from game loop → network thread
pub enum NetCmd {
    SendCursor { x: f32, y: f32 },
    SendState { 
        money: i64, 
        inventory: HashMap<String, i32>, 
        production_rate: f64,
        x: f32,
        y: f32,
        grid: Option<String>,
        seed: u64,
    },
    #[allow(dead_code)]
    SendChat { msg: String },
    #[allow(dead_code)]
    Disconnect,
}

/// Events from network thread → game loop
#[derive(Debug)]
pub enum NetEvent {
    Connected { 
        money: i64, 
        inventory: HashMap<String, i32>, 
        offline_earnings: i64, 
        region: String,
        grid: Option<String>,
        x: f32,
        y: f32,
        seed: u64,
    },
    MarketUpdate { item: String, price: i64 },
    CursorFrom { #[allow(dead_code)] sender: String, x: f32, y: f32 },
    ChatFrom { #[allow(dead_code)] sender: String, #[allow(dead_code)] msg: String },
    PlayerList(Vec<String>),
    NpcUpdate(Vec<NpcInfo>),
    StateSync {
        sender: String,
        money: i64,
        inventory: HashMap<String, i32>,
        grid: Option<String>,
        x: f32,
        y: f32,
    },
    Disconnected(String),
    #[allow(dead_code)]
    Error(String),
}

#[derive(Debug, Clone)]
pub struct NpcInfo {
    pub id: String,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub state: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Packet {
    #[serde(rename = "type")]
    ptype: String,
    data: serde_json::Value,
    #[serde(default)]
    sender: String,
}

#[derive(Deserialize, Debug)]
struct RegionStateData {
    money: i64,
    inventory: HashMap<String, i32>,
    offline_earnings: i64,
    region: String,
    grid: Option<serde_json::Value>,
    x: f32,
    y: f32,
    seed: u64,
}

#[derive(Deserialize, Debug)]
struct MarketEventData {
    item: String,
    price: i64,
}

#[derive(Deserialize, Debug)]
struct CursorSyncData {
    x: f32,
    y: f32,
}

#[derive(Deserialize, Debug)]
struct NpcUpdateData {
    npcs: Vec<NpcInfoRaw>,
}

#[derive(Deserialize, Debug)]
struct NpcInfoRaw {
    id: String,
    name: String,
    x: f32,
    y: f32,
    state: String,
}

/// Login to the server, get a JWT token
pub fn login(server_url: &str, username: &str, password: &str) -> Result<(String, String), String> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/login", server_url);

    #[derive(Serialize)]
    struct Creds { username: String, password: String }
    #[derive(Deserialize)]
    struct LoginResp { token: String, username: String }

    let resp = client.post(&url)
        .json(&Creds { username: username.to_string(), password: password.to_string() })
        .send()
        .map_err(|e| format!("Conexão falhou: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Login falhou: {}", resp.status()));
    }

    let data: LoginResp = resp.json().map_err(|e| format!("Parse erro: {}", e))?;
    Ok((data.token, data.username))
}

/// Register a new account
pub fn register(server_url: &str, username: &str, password: &str) -> Result<(), String> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/register", server_url);

    #[derive(Serialize)]
    struct Creds { username: String, password: String }

    let resp = client.post(&url)
        .json(&Creds { username: username.to_string(), password: password.to_string() })
        .send()
        .map_err(|e| format!("Conexão falhou: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Registro falhou: {}", resp.status()));
    }
    Ok(())
}

/// Fetch server list
pub fn fetch_servers(server_url: &str) -> Vec<ServerInfo> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/servers", server_url);
    match client.get(&url).send() {
        Ok(resp) => resp.json().unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct ServerInfo {
    pub name: String,
    pub region: String,
    #[allow(dead_code)]
    pub port: serde_json::Value,
    pub players: i32,
    pub max: i32,
}

/// Spawn the network thread. Returns channels for bidirectional communication.
pub fn connect_ws(
    server_url: &str,
    token: &str,
    region: &str,
) -> Result<(mpsc::Sender<NetCmd>, mpsc::Receiver<NetEvent>), String> {
    // Build WS URL
    let ws_url = server_url.replace("http://", "ws://").replace("https://", "wss://");
    let full_url = format!("{}/ws?token={}&region={}", ws_url, token, region);

    let (socket, _response) = connect(&full_url)
        .map_err(|e| format!("WebSocket connect failed: {}", e))?;

    let (cmd_tx, cmd_rx) = mpsc::channel::<NetCmd>();
    let (evt_tx, evt_rx) = mpsc::channel::<NetEvent>();

    // Spawn network IO thread
    std::thread::spawn(move || {
        run_ws_loop(socket, cmd_rx, evt_tx);
    });

    Ok((cmd_tx, evt_rx))
}

fn run_ws_loop(
    mut ws: WebSocket<MaybeTlsStream<TcpStream>>,
    cmd_rx: mpsc::Receiver<NetCmd>,
    evt_tx: mpsc::Sender<NetEvent>,
) {
    // Make socket non-blocking for interleaved read/write
    if let MaybeTlsStream::Plain(ref s) = ws.get_ref() {
        let _ = s.set_nonblocking(true);
    }

    loop {
        // 1. Process outgoing commands
        while let Ok(cmd) = cmd_rx.try_recv() {
            let pkt_json = match cmd {
                NetCmd::SendCursor { x, y } => {
                    let data = serde_json::json!({"x": x, "y": y});
                    serde_json::json!({"type": "cursor_sync", "data": data})
                }
                NetCmd::SendState { money, inventory, production_rate, x, y, grid, seed } => {
                    let mut data = serde_json::json!({
                        "tick": 0,
                        "money": money,
                        "inventory": inventory,
                        "production_rate": production_rate,
                        "x": x,
                        "y": y,
                        "seed": seed,
                    });
                    if let Some(g) = grid {
                        if let Ok(g_val) = serde_json::from_str::<serde_json::Value>(&g) {
                            data["grid"] = g_val;
                        }
                    }
                    serde_json::json!({"type": "state_update", "data": data})
                }
                NetCmd::SendChat { msg } => {
                    let data = serde_json::json!({"msg": msg});
                    serde_json::json!({"type": "chat", "data": data})
                }
                NetCmd::Disconnect => {
                    let _ = ws.close(None);
                    let _ = evt_tx.send(NetEvent::Disconnected("Manual".into()));
                    return;
                }
            };

            let msg = Message::Text(pkt_json.to_string());
            let mut sent = false;
            for _ in 0..5 { // Retry a few times if WouldBlock
                match ws.send(msg.clone()) {
                    Ok(_) => { sent = true; break; }
                    Err(tungstenite::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(std::time::Duration::from_millis(5));
                        continue;
                    }
                    Err(e) => {
                        let _ = evt_tx.send(NetEvent::Error(format!("Send failed: {}", e)));
                        break;
                    }
                }
            }
            if !sent {
                let _ = evt_tx.send(NetEvent::Error("Send timed out (WouldBlock)".into()));
            }
        }

        // 2. Read incoming messages (non-blocking)
        match ws.read() {
            Ok(Message::Text(text)) => {
                if let Ok(pkt) = serde_json::from_str::<Packet>(&text) {
                    match pkt.ptype.as_str() {
                        "region_state" => {
                            if let Ok(rs) = serde_json::from_value::<RegionStateData>(pkt.data) {
                                let grid_str = rs.grid.map(|v| v.to_string());
                                let _ = evt_tx.send(NetEvent::Connected {
                                    money: rs.money,
                                    inventory: rs.inventory,
                                    offline_earnings: rs.offline_earnings,
                                    region: rs.region,
                                    grid: grid_str,
                                    x: rs.x,
                                    y: rs.y,
                                    seed: rs.seed,
                                });
                            }
                        }
                        "market_event" => {
                            if let Ok(me) = serde_json::from_value::<MarketEventData>(pkt.data) {
                                let _ = evt_tx.send(NetEvent::MarketUpdate { item: me.item, price: me.price });
                            }
                        }
                        "state_update" => {
                            if let Ok(rs) = serde_json::from_value::<RegionStateData>(pkt.data) {
                                let grid_str = rs.grid.map(|v| v.to_string());
                                let _ = evt_tx.send(NetEvent::StateSync {
                                    sender: pkt.sender,
                                    money: rs.money,
                                    inventory: rs.inventory,
                                    grid: grid_str,
                                    x: rs.x,
                                    y: rs.y,
                                });
                            }
                        }
                        "cursor_sync" => {
                            if let Ok(cs) = serde_json::from_value::<CursorSyncData>(pkt.data) {
                                let _ = evt_tx.send(NetEvent::CursorFrom { sender: pkt.sender, x: cs.x, y: cs.y });
                            }
                        }
                        "chat" => {
                            #[derive(Deserialize)]
                            struct CD { msg: String }
                            if let Ok(cd) = serde_json::from_value::<CD>(pkt.data) {
                                let _ = evt_tx.send(NetEvent::ChatFrom { sender: pkt.sender, msg: cd.msg });
                            }
                        }
                        "player_list" => {
                            if let Ok(names) = serde_json::from_value::<Vec<String>>(pkt.data) {
                                let _ = evt_tx.send(NetEvent::PlayerList(names));
                            }
                        }
                        "npc_update" => {
                            if let Ok(nud) = serde_json::from_value::<NpcUpdateData>(pkt.data) {
                                let npcs = nud.npcs.into_iter().map(|n| NpcInfo {
                                    id: n.id,
                                    name: n.name,
                                    x: n.x,
                                    y: n.y,
                                    state: n.state,
                                }).collect();
                                let _ = evt_tx.send(NetEvent::NpcUpdate(npcs));
                            }
                        }
                        _ => {}
                    }
                }
            }
            Ok(Message::Close(_)) => {
                let _ = evt_tx.send(NetEvent::Disconnected("Server closed".into()));
                return;
            }
            Err(tungstenite::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No data available — sleep briefly to avoid busy spin
                std::thread::sleep(std::time::Duration::from_millis(16));
            }
            Err(_) => {
                // Connection lost
                let _ = evt_tx.send(NetEvent::Disconnected("Connection lost".into()));
                return;
            }
            _ => {}
        }
    }
}
