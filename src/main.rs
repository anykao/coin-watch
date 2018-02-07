#![feature(proc_macro, conservative_impl_trait, generators)]

extern crate websocket;
extern crate futures_await as futures;
extern crate tokio_core;
#[macro_use]
extern crate serde_derive;
extern crate serde_json as json;
extern crate termion;

use tokio_core::reactor::{Core, Handle};
use futures::sink::Sink;
use futures::stream::Stream;
use futures::prelude::*;
use websocket::result::WebSocketError;
use websocket::{ClientBuilder, OwnedMessage};
use websocket::Message;
use std::io::{Write, stdout};

const CONNECTION: &'static str = "wss://ws-api.coincheck.com/";

#[derive(Serialize)]
struct WsRequest {
    #[serde(rename = "type")] type_: String,
    channel: String,
}

#[derive(Deserialize)]
struct WsResponse(String, Ita);

#[derive(Deserialize)]
struct Pair(pub String, pub String);

#[derive(Deserialize)]
struct Ita {
    asks: Vec<Pair>,
    bids: Vec<Pair>,
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle: Handle = core.handle();
    let runner = run(handle);
    core.run(runner).unwrap();
}

#[async]
fn run(h: Handle) -> Result<(), WebSocketError> {
    let stdout = stdout();
    let (duplex, _) = await!(ClientBuilder::new(CONNECTION).unwrap().async_connect_secure(None, &h)).unwrap();
    let (sink, stream) = duplex.split();
    let request = WsRequest {
        type_: String::from("subscribe"),
        channel: String::from("btc_jpy-orderbook"),
    };
    let msg = json::to_string(&request).unwrap();
    await!(sink.send(Message::text(msg).into())).unwrap();
    #[async]
        for message in stream {
        let mut stdout = stdout.lock();
        match message {
            OwnedMessage::Text(msg) => {
                let res: WsResponse = json::from_str(&msg).unwrap();
                let ita = res.1;
                write!(stdout, "{}{}{}", termion::cursor::Hide, termion::clear::All, termion::cursor::Goto(1, 1)).unwrap();
                writeln!(stdout, "{:15}{:15}{:15}{:15}", "Ask", "Quantity", "Bid", "Quantity").unwrap();
                for (ask, bid) in ita.asks.iter().zip(&ita.bids) {
                    writeln!(stdout, "{:15}{:15}{:15}{:15}", ask.0, ask.1, bid.0, bid.1).unwrap();
                }
                stdout.flush().unwrap();
            }
            _ => {}
        };
    }
    Ok(())
}

