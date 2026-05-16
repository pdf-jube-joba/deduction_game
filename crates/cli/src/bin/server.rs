use cli::{host::GameHost, ErrorResponse, MoveRequest};
use game_core::config::default_config;
use game_core::defs::Move;
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

fn main() {
    let port = std::env::args()
        .nth(1)
        .unwrap_or_else(|| usage_and_exit("server <port>"))
        .parse::<u16>()
        .unwrap_or_else(|_| usage_and_exit("server <port>"));

    let host = Arc::new(Mutex::new(GameHost::new(default_config())));
    let listener = TcpListener::bind(("127.0.0.1", port)).expect("failed to bind port");
    println!("listening on http://127.0.0.1:{port}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let host = Arc::clone(&host);
                std::thread::spawn(move || {
                    if let Err(err) = handle_connection(stream, host) {
                        eprintln!("{err}");
                    }
                });
            }
            Err(err) => eprintln!("{err}"),
        }
    }
}

fn handle_connection(stream: TcpStream, host: Arc<Mutex<GameHost>>) -> Result<(), String> {
    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .map_err(|err| err.to_string())?;
    if request_line.trim().is_empty() {
        return Ok(());
    }

    let mut content_length = 0usize;
    let mut player_secret: Option<String> = None;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).map_err(|err| err.to_string())?;
        if line == "\r\n" || line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse::<usize>().unwrap_or(0);
            }
            if name.eq_ignore_ascii_case("x-player-secret") {
                player_secret = Some(value.trim().to_string());
            }
        }
    }

    let mut body = vec![0; content_length];
    reader
        .read_exact(&mut body)
        .map_err(|err| err.to_string())?;
    let mut stream = reader.into_inner();

    let mut parts = request_line.split_whitespace();
    let method = parts.next().ok_or("missing method")?;
    let path = parts.next().ok_or("missing path")?;
    let peer = stream
        .peer_addr()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|_| "<unknown>".to_string());

    match (method, path) {
        ("POST", "/join") => {
            let mut host = host.lock().map_err(|_| "lock poisoned".to_string())?;
            let Some(response) = host.join() else {
                eprintln!("[join/reject] peer={peer} reason=all_players_joined");
                return write_error(&mut stream, 409, "all players are already joined");
            };
            eprintln!(
                "[join] peer={peer} player={} secret={}",
                response.player,
                short_secret(&response.secret)
            );
            write_json(&mut stream, 200, &response)
        }
        ("POST", "/move") => {
            let request: MoveRequest =
                serde_json::from_slice(&body).map_err(|err| err.to_string())?;
            let Some(secret) = player_secret.as_deref() else {
                eprintln!("[move/reject] peer={peer} reason=missing_secret");
                return write_error(&mut stream, 401, "missing player secret");
            };
            let mut host = host.lock().map_err(|_| "lock poisoned".to_string())?;
            let action = request.action;
            match host.apply_move(secret, action.clone()) {
                Ok(response) => {
                    eprintln!(
                        "[move] peer={peer} secret={} action={}",
                        short_secret(secret),
                        format_move(&action)
                    );
                    if let Some(winner) = &response.winner {
                        eprintln!("[win] winner={winner:?}");
                    }
                    write_json(&mut stream, 200, &response)
                }
                Err(err) => {
                    eprintln!(
                        "[move/reject] peer={peer} secret={} action={} reason={err}",
                        short_secret(secret),
                        format_move(&action)
                    );
                    write_error(&mut stream, 401, &err)
                }
            }
        }
        ("GET", "/state") => {
            let Some(secret) = player_secret.as_deref() else {
                eprintln!("[state/reject] peer={peer} reason=missing_secret");
                return write_error(&mut stream, 401, "missing player secret");
            };
            let host = host.lock().map_err(|_| "lock poisoned".to_string())?;
            match host.state_for_secret(secret) {
                Some(response) => write_json(&mut stream, 200, &response),
                None => {
                    eprintln!(
                        "[state/reject] peer={peer} secret={} reason=invalid_secret",
                        short_secret(secret)
                    );
                    write_error(&mut stream, 401, "invalid secret")
                }
            }
        }
        _ => {
            eprintln!("[request/reject] peer={peer} method={method} path={path} reason=not_found");
            write_error(&mut stream, 404, "not found")
        }
    }
}

fn write_json<T: serde::Serialize>(
    stream: &mut TcpStream,
    status: u16,
    value: &T,
) -> Result<(), String> {
    let body = serde_json::to_vec(value).map_err(|err| err.to_string())?;
    write_response(stream, status, "application/json", &body)
}

fn write_error(stream: &mut TcpStream, status: u16, error: &str) -> Result<(), String> {
    let body = serde_json::to_vec(&ErrorResponse {
        error: error.to_string(),
    })
    .map_err(|err| err.to_string())?;
    write_response(stream, status, "application/json", &body)
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> Result<(), String> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        409 => "Conflict",
        _ => "Internal Server Error",
    };
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .map_err(|err| err.to_string())?;
    stream.write_all(body).map_err(|err| err.to_string())?;
    stream.flush().map_err(|err| err.to_string())
}

fn usage_and_exit(message: &str) -> ! {
    eprintln!("usage: {message}");
    std::process::exit(2);
}

fn short_secret(secret: &str) -> &str {
    let len = secret.len().min(8);
    &secret[..len]
}

fn format_move(action: &Move) -> String {
    match action {
        Move::Query {
            query_to,
            query_sort,
        } => format!("query(to={query_to}, sort={query_sort})"),
        Move::Declare { declare } => format!("declare({declare:?})"),
    }
}
