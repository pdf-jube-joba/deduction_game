use game_core::defs::{Info, Move, MoveAns};
use serde::{Deserialize, Serialize};
use std::{
    io::{self, BufRead, BufReader, Read, Write},
    net::TcpStream,
};

pub mod host;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateResponse {
    pub you: usize,
    pub current_turn: usize,
    pub your_turn: bool,
    pub winner: Option<Vec<usize>>,
    pub info: Info,
    pub possible_moves: Vec<Move>,
    pub history: Vec<String>,
}

impl StateResponse {
    pub fn from_info(
        you: usize,
        current_turn: usize,
        winner: Option<Vec<usize>>,
        info: Info,
        possible_moves: Vec<Move>,
    ) -> Self {
        let your_turn = you == current_turn && winner.is_none();
        let history = info.query_answer.iter().map(format_move_ans).collect();
        Self {
            you,
            current_turn,
            your_turn,
            winner,
            info,
            history,
            possible_moves,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinResponse {
    pub player: usize,
    pub player_num: usize,
    pub secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveRequest {
    pub action: Move,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveResponse {
    pub accepted: bool,
    pub winner: Option<Vec<usize>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub fn format_move_ans(value: &MoveAns) -> String {
    match value {
        MoveAns::Query {
            who,
            query_to,
            query_sort,
            ans,
        } => format!("player {who} asked player {query_to} about {query_sort}: {ans}"),
        MoveAns::Declare { who, declare, ans } => {
            format!("player {who} declared {declare:?}: {ans}")
        }
    }
}

pub fn get_json<T: for<'de> Deserialize<'de>>(
    port: u16,
    path: &str,
    secret: Option<&str>,
) -> io::Result<T> {
    request_json("GET", port, path, None, secret)
}

pub fn post_json<B: Serialize, T: for<'de> Deserialize<'de>>(
    port: u16,
    path: &str,
    body: &B,
    secret: Option<&str>,
) -> io::Result<T> {
    let body = serde_json::to_vec(body)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;
    request_json("POST", port, path, Some(&body), secret)
}

fn request_json<T: for<'de> Deserialize<'de>>(
    method: &str,
    port: u16,
    path: &str,
    body: Option<&[u8]>,
    secret: Option<&str>,
) -> io::Result<T> {
    let mut stream = TcpStream::connect(("127.0.0.1", port))?;
    let body = body.unwrap_or(&[]);
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    )?;
    if let Some(secret) = secret {
        write!(stream, "X-Player-Secret: {secret}\r\n")?;
    }
    write!(stream, "\r\n")?;
    if !body.is_empty() {
        stream.write_all(body)?;
    }
    stream.flush()?;

    let mut reader = BufReader::new(stream);
    let mut status_line = String::new();
    reader.read_line(&mut status_line)?;
    let status_code = status_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing status"))?
        .parse::<u16>()
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;

    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        if line == "\r\n" || line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse::<usize>().unwrap_or(0);
            }
        }
    }

    let mut body = vec![0; content_length];
    reader.read_exact(&mut body)?;
    if (200..300).contains(&status_code) {
        serde_json::from_slice(&body)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))
    } else {
        let error = serde_json::from_slice::<ErrorResponse>(&body)
            .map(|v| v.error)
            .unwrap_or_else(|_| String::from_utf8_lossy(&body).into_owned());
        Err(io::Error::other(format!(
            "server returned {status_code}: {error}"
        )))
    }
}
