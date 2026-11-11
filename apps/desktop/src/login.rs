use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

/// The event the window listens to for the sign-in answer.
pub const LOGIN_EVENT: &str = "toile://login";

const WAIT: Duration = Duration::from_secs(300);
const POLL: Duration = Duration::from_millis(200);
const PAGE: &str = "<!doctype html><html lang=\"fr\"><meta charset=\"utf-8\"><title>Toile</title><body style=\"font-family:system-ui;display:grid;place-items:center;height:100vh;margin:0\"><p>Connexion terminée. Vous pouvez revenir dans Toile.</p></body></html>";

/// What porte's redirect to the loopback carried.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Answer {
    /// The one-time login code.
    pub code: String,
    /// The `cli_state` the window sent, echoed back.
    pub state: String,
    /// Why the sign-in stopped, when it did.
    pub error: String,
}

/// Reads the answer out of an HTTP request line such as `GET /?code=x&state=y HTTP/1.1`.
/// Anything else (a favicon fetch, a malformed line) is `None`.
pub fn parse_answer(request_line: &str) -> Option<Answer> {
    let mut parts = request_line.split_whitespace();
    if parts.next()? != "GET" {
        return None;
    }
    let url = url::Url::parse(&format!("http://127.0.0.1{}", parts.next()?)).ok()?;
    if url.path() != "/" {
        return None;
    }
    let mut answer = Answer::default();
    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "code" => answer.code = value.into_owned(),
            "state" => answer.state = value.into_owned(),
            "error" => answer.error = value.into_owned(),
            _ => {}
        }
    }
    (!answer.code.is_empty() || !answer.error.is_empty()).then_some(answer)
}

fn read(stream: &mut TcpStream) -> Option<Answer> {
    stream.set_nonblocking(false).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok()?;
    let mut line = String::new();
    BufReader::new(&*stream).read_line(&mut line).ok()?;
    let answer = parse_answer(&line);
    let (status, body) = if answer.is_some() {
        ("200 OK", PAGE)
    } else {
        ("404 Not Found", "")
    };
    let _ = write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    answer
}

fn wait(listener: &TcpListener) -> Answer {
    let deadline = Instant::now() + WAIT;
    while Instant::now() < deadline {
        match listener.accept() {
            Ok((mut stream, _)) => {
                if let Some(answer) = read(&mut stream) {
                    return answer;
                }
            }
            Err(_) => thread::sleep(POLL),
        }
    }
    Answer {
        error: "La connexion a expiré. Réessayez.".to_owned(),
        ..Answer::default()
    }
}

/// Opens a one-shot listener on 127.0.0.1 and returns its port. The first request carrying a
/// code or an error, or five minutes of silence, is emitted as [`LOGIN_EVENT`].
#[tauri::command]
pub fn login_listen(app: AppHandle) -> Result<u16, String> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|err| err.to_string())?;
    listener
        .set_nonblocking(true)
        .map_err(|err| err.to_string())?;
    let port = listener.local_addr().map_err(|err| err.to_string())?.port();
    thread::spawn(move || {
        let answer = wait(&listener);
        if let Err(err) = app.emit(LOGIN_EVENT, answer) {
            eprintln!("toile: could not deliver the sign-in answer: {err}");
        }
    });
    Ok(port)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_code_and_state() {
        let answer = parse_answer("GET /?code=abc&state=s_1 HTTP/1.1\r\n").unwrap();
        assert_eq!(
            answer,
            Answer {
                code: "abc".into(),
                state: "s_1".into(),
                error: String::new()
            }
        );
    }

    #[test]
    fn ignores_other_requests() {
        assert_eq!(parse_answer("GET /favicon.ico HTTP/1.1"), None);
        assert_eq!(parse_answer("GET / HTTP/1.1"), None);
        assert_eq!(parse_answer("POST /?code=abc HTTP/1.1"), None);
        assert_eq!(parse_answer(""), None);
    }

    #[test]
    fn carries_an_error() {
        assert_eq!(
            parse_answer("GET /?error=denied HTTP/1.1").unwrap().error,
            "denied"
        );
    }

    #[test]
    fn a_real_request_reaches_the_listener() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        listener.set_nonblocking(true).unwrap();
        let port = listener.local_addr().unwrap().port();
        let client = thread::spawn(move || {
            let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
            stream
                .write_all(b"GET /?code=c0de&state=st HTTP/1.1\r\nHost: x\r\n\r\n")
                .unwrap();
            let mut reply = String::new();
            BufReader::new(stream).read_line(&mut reply).unwrap();
            reply
        });
        let answer = wait(&listener);
        assert_eq!(answer.code, "c0de");
        assert!(client.join().unwrap().starts_with("HTTP/1.1 200"));
    }
}
