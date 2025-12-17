use crate::{game::random_shape, CHUNK_SIZE};
use crate::game::Piece;

#[allow(dead_code)]
pub fn command_to_pieces(cmd: &str) -> Vec<Piece> {
    let mut pieces = Vec::new();
    for chunk in command_to_chunks(cmd) {
        let payload = chunk_to_payload(&chunk);
        pieces.push(Piece::with_payload(random_shape(), payload));
    }
    pieces
}

pub fn command_to_chunks(cmd: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    for token in crate::commands::tokenize_command(cmd) {
        chunks.extend(chunk_token(&token));
    }
    chunks
}

pub fn chunk_token(token: &str) -> Vec<String> {
    let mut res = Vec::new();
    let mut chars: Vec<char> = token.chars().collect();
    while !chars.is_empty() {
        let mut take: Vec<char> = chars.drain(..CHUNK_SIZE.min(chars.len())).collect();
        if take.len() < CHUNK_SIZE {
            take.resize(CHUNK_SIZE, '░');
        }
        res.push(take.into_iter().collect());
    }
    if res.is_empty() {
        res.push("░░░░░░░░".to_string());
    }
    res
}

pub fn chunk_to_payload(chunk: &str) -> Vec<char> {
    let mut chars: Vec<char> = chunk.chars().collect();
    if chars.len() < CHUNK_SIZE {
        chars.resize(CHUNK_SIZE, '░');
    }
    chars.truncate(CHUNK_SIZE);
    chars
}
