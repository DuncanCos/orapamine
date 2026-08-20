//! Test d'intégration bout-en-bout : deux clients WebSocket scriptent une
//! partie complète (création, jonction, placement, quelques actions de
//! jeu, soumission de solution correcte), plus un test dédié de la
//! reconnexion. Utilise `tokio-tungstenite` comme client de test, contre
//! le vrai routeur axum du serveur lancé sur un port éphémère local.

use futures_util::{SinkExt, StreamExt};
use orapa_core::{random_valid_placement, GameOptions, PieceCatalog};
use orapa_server::state::AppState;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use serde_json::{json, Value};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

async fn spawn_server() -> SocketAddr {
    let state = AppState::new();
    let app = orapa_server::build_router(state, "web/dist_test_does_not_exist");
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

type WsStream = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn connect(addr: SocketAddr) -> WsStream {
    let url = format!("ws://{addr}/ws");
    let (ws, _) = tokio_tungstenite::connect_async(url).await.unwrap();
    ws
}

async fn send(ws: &mut WsStream, msg: Value) {
    ws.send(Message::Text(msg.to_string())).await.unwrap();
}

/// Reçoit le prochain message texte, en filtrant les `StateUpdate`
/// (bruyants et envoyés après chaque action) quand `skip_state_updates` est
/// vrai — pratique pour aller droit au message qui nous intéresse.
async fn recv_matching(ws: &mut WsStream, msg_type: &str) -> Value {
    loop {
        let msg = ws.next().await.expect("connexion fermée").unwrap();
        if let Message::Text(text) = msg {
            let v: Value = serde_json::from_str(&text).unwrap();
            if v["type"] == msg_type {
                return v;
            }
        }
    }
}

fn valid_placement_json(catalog: &PieceCatalog, seed: u64) -> Value {
    let mut rng = SmallRng::seed_from_u64(seed);
    let placement = random_valid_placement(&mut rng, catalog, GameOptions::default());
    serde_json::to_value(placement).unwrap()
}

#[tokio::test]
async fn full_duel_game_via_websocket() {
    let addr = spawn_server().await;
    let catalog = PieceCatalog::default_catalog();

    let mut a = connect(addr).await;
    let mut b = connect(addr).await;

    send(
        &mut a,
        json!({"type": "CreateGame", "pseudo": "Alice", "options": {"diamond": false, "black": false, "lives": 1, "turn_timer_secs": null, "help_mode": true}}),
    )
    .await;
    let created = recv_matching(&mut a, "GameCreated").await;
    let code = created["code"].as_str().unwrap().to_string();

    send(
        &mut b,
        json!({"type": "JoinGame", "code": code, "pseudo": "Bob"}),
    )
    .await;
    let joined = recv_matching(&mut b, "Joined").await;
    assert_eq!(joined["player_index"], 1);

    // Les deux voient la partie passer en phase "placement" une fois à deux
    // (le tout premier `StateUpdate` reçu par `a`, juste après sa propre
    // création de partie, est encore en phase "lobby" — on saute jusqu'au
    // bon).
    let mut state_a = recv_matching(&mut a, "StateUpdate").await;
    while state_a["phase"] == "lobby" {
        state_a = recv_matching(&mut a, "StateUpdate").await;
    }
    assert_eq!(state_a["phase"], "placement");

    // Chacun pose une disposition valide (générée par le moteur, hors
    // protocole, pour garantir qu'elle respecte les 6 contraintes) et se
    // déclare prêt.
    let placements = [
        valid_placement_json(&catalog, 10),
        valid_placement_json(&catalog, 20),
    ];
    send(&mut a, json!({"type": "SetPlacement", "pieces": placements[0].clone()})).await;
    send(&mut b, json!({"type": "SetPlacement", "pieces": placements[1].clone()})).await;
    send(&mut a, json!({"type": "ReadyToPlay"})).await;
    send(&mut b, json!({"type": "ReadyToPlay"})).await;

    // `players[i]` est le socket du joueur d'index i ; on route chaque
    // action vers `players[current_turn]` d'après le dernier `StateUpdate`
    // reçu, sans jamais avoir à deviner qui a la main.
    let mut players = [a, b];

    // Une fois les deux prêts, la partie passe en "playing" : on lit les
    // `StateUpdate` côté a jusqu'à trouver celui qui porte le tour actif.
    let mut current_turn = 0usize;
    for _ in 0..10 {
        let s = recv_matching(&mut players[0], "StateUpdate").await;
        if s["phase"] == "playing" {
            current_turn = s["current_turn"].as_u64().unwrap() as usize;
            break;
        }
    }
    // Idem côté b, pour vider son propre flux de `StateUpdate` avant de
    // scripter la suite.
    for _ in 0..10 {
        let s = recv_matching(&mut players[1], "StateUpdate").await;
        if s["phase"] == "playing" {
            break;
        }
    }

    // Le joueur actif tire une onde.
    send(&mut players[current_turn], json!({"type": "FireBeam", "entry": "A"})).await;
    let beam_result = recv_matching(&mut players[current_turn], "BeamResult").await;
    assert!(beam_result["outcome"]["kind"].is_string());
    let handoff = recv_matching(&mut players[current_turn], "StateUpdate").await;
    current_turn = handoff["current_turn"].as_u64().unwrap() as usize;

    // Puis l'autre joueur sonde une case.
    send(&mut players[current_turn], json!({"type": "Probe", "x": 0, "y": 0})).await;
    let probe_result = recv_matching(&mut players[current_turn], "ProbeResult").await;
    assert!(probe_result["result"].is_string() || probe_result["result"].is_object());
    let handoff = recv_matching(&mut players[current_turn], "StateUpdate").await;
    current_turn = handoff["current_turn"].as_u64().unwrap() as usize;

    // Le joueur actif soumet la bonne disposition adverse : `correct` doit
    // être vrai quel que soit qui a joué en premier (la règle de mort
    // subite, elle, ne change que la suite — victoire immédiate ou dernier
    // tour de l'adversaire —, pas la valeur de `correct` elle-même ;
    // couverte séparément dans les tests unitaires de `logic`).
    let opponent_index = 1 - current_turn;
    send(
        &mut players[current_turn],
        json!({"type": "SubmitSolution", "pieces": placements[opponent_index].clone()}),
    )
    .await;
    let solution_result = recv_matching(&mut players[current_turn], "SolutionResult").await;
    assert_eq!(solution_result["correct"], true);
}

#[tokio::test]
async fn reconnect_restores_player_state() {
    let addr = spawn_server().await;

    let mut a = connect(addr).await;
    send(
        &mut a,
        json!({"type": "CreateGame", "pseudo": "Alice", "options": {"diamond": false, "black": false, "lives": 1, "turn_timer_secs": null, "help_mode": false}}),
    )
    .await;
    let created = recv_matching(&mut a, "GameCreated").await;
    let token = created["token"].as_str().unwrap().to_string();

    // Alice ferme sa connexion (simulate un rechargement de page)...
    a.close(None).await.ok();
    drop(a);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // ...puis se reconnecte avec son jeton et retrouve son état.
    let mut a2 = connect(addr).await;
    send(&mut a2, json!({"type": "Reconnect", "token": token})).await;
    let state = recv_matching(&mut a2, "StateUpdate").await;
    assert_eq!(state["your_index"], 0);
    assert_eq!(state["players"][0]["pseudo"], "Alice");
    assert_eq!(state["players"][0]["connected"], true);
}

#[tokio::test]
async fn solo_mode_lets_a_single_player_start_immediately() {
    let addr = spawn_server().await;
    let mut a = connect(addr).await;
    send(
        &mut a,
        json!({"type": "StartSolo", "pseudo": "Solo", "options": {"diamond": false, "black": false, "lives": 1, "turn_timer_secs": null, "help_mode": false}}),
    )
    .await;
    let created = recv_matching(&mut a, "GameCreated").await;
    assert_eq!(created["player_index"], 0);

    let state = recv_matching(&mut a, "StateUpdate").await;
    assert_eq!(state["phase"], "playing");

    send(&mut a, json!({"type": "FireBeam", "entry": "1"})).await;
    let beam_result = recv_matching(&mut a, "BeamResult").await;
    assert!(beam_result["outcome"]["kind"].is_string());
}
