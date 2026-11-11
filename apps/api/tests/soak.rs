mod common;

use std::time::{Duration, Instant};

use common::board::rect;
use common::fixture;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message as Frame;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use yrs::Transact;
use yrs::sync::{Message, SyncMessage};
use yrs::updates::encoder::Encode;

const CLIENTS: usize = 50;
const ROUNDS: usize = 200;

type Sink = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    Frame,
>;

fn is_update(bytes: &[u8]) -> bool {
    let mut decoder = yrs::updates::decoder::DecoderV1::from(bytes);
    yrs::sync::protocol::MessageReader::new(&mut decoder)
        .filter_map(Result::ok)
        .any(|m| matches!(m, Message::Sync(SyncMessage::Update(_))))
}

async fn join(url: &str, token: &str, index: usize, arrivals: mpsc::UnboundedSender<(usize, Instant)>) -> Sink {
    let mut request = url.into_client_request().unwrap();
    request.headers_mut().insert("authorization", format!("Bearer {token}").parse().unwrap());
    let (socket, _) = tokio_tungstenite::connect_async(request).await.unwrap();
    let (mut sink, mut stream) = socket.split();
    let hello = Message::Sync(SyncMessage::SyncStep1(Default::default())).encode_v1();
    sink.send(Frame::Binary(hello.into())).await.unwrap();
    tokio::spawn(async move {
        while let Some(Ok(frame)) = stream.next().await {
            record(&frame, index, &arrivals);
        }
    });
    sink
}

fn record(frame: &Frame, index: usize, arrivals: &mpsc::UnboundedSender<(usize, Instant)>) {
    if matches!(frame, Frame::Binary(bytes) if is_update(bytes)) {
        _ = arrivals.send((index, Instant::now()));
    }
}

fn percentile(sorted: &[Duration], p: f64) -> Duration {
    sorted[((sorted.len() as f64 - 1.0) * p).round() as usize]
}

fn frame(round: usize) -> Vec<u8> {
    let doc = api::modules::board::new_doc();
    let shapes = doc.get_or_insert_map("shapes");
    let update = {
        let mut txn = doc.transact_mut();
        rect(&mut txn, &shapes, &format!("r{round}"), round as f64);
        txn.encode_update_v1()
    };
    Message::Sync(SyncMessage::Update(update)).encode_v1()
}

async fn fan_out(
    sinks: &mut [Sink],
    arrivals: &mut mpsc::UnboundedReceiver<(usize, Instant)>,
    round: usize,
) -> Vec<Duration> {
    let frame = frame(round);
    let sent = Instant::now();
    sinks[round % CLIENTS].send(Frame::Binary(frame.into())).await.unwrap();
    let mut latencies = Vec::with_capacity(CLIENTS - 1);
    for _ in 1..CLIENTS {
        let (_, at) = tokio::time::timeout(Duration::from_secs(5), arrivals.recv()).await.unwrap().unwrap();
        latencies.push(at - sent);
    }
    latencies
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "soak: run with --ignored --nocapture"]
async fn fifty_clients_fan_out_every_update_under_100_ms() {
    let Some(pool) = common::pool("toile_soak").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::board(&app, &roles.member, None, "Soak").await);
    let url = common::board::ws(&app.serve().await, &id);
    let (tx, mut arrivals) = mpsc::unbounded_channel();
    let mut sinks = Vec::new();
    for index in 0..CLIENTS {
        sinks.push(join(&url, &roles.member.token, index, tx.clone()).await);
    }
    tokio::time::sleep(Duration::from_millis(500)).await;
    let mut latencies = Vec::with_capacity(ROUNDS * (CLIENTS - 1));
    for round in 0..ROUNDS {
        latencies.extend(fan_out(&mut sinks, &mut arrivals, round).await);
    }
    latencies.sort();
    let (p50, p95, p99, max) = (
        percentile(&latencies, 0.5),
        percentile(&latencies, 0.95),
        percentile(&latencies, 0.99),
        latencies[latencies.len() - 1],
    );
    println!(
        "soak: {CLIENTS} clients, {ROUNDS} updates, {} deliveries: p50 {p50:?} p95 {p95:?} p99 {p99:?} max {max:?}",
        latencies.len()
    );
    assert!(p99 < Duration::from_millis(100), "p99 fan-out {p99:?}");
}
