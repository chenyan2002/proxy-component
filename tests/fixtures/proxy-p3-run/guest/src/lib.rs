//! A main component of `tests/fixtures/proxy-p3`, which `make test-fuzz` fuzzes: it uses each
//! kind of async function, stream, future and error-context of the WIT. It tolerates whatever a
//! host may do, e.g. stop reading a stream it returned, so that a fuzzed run doesn't trap.
wit_bindgen::generate!({
    path: "../../proxy-p3",
    world: "root",
    generate_all,
});

use exports::test::p3::api::Guest;
use test::p3::host::{self, Body, ErrorCode};
use wit_bindgen::rt::async_support::ErrorContext;
use wit_bindgen::{StreamReader, StreamResult};

struct Component;

impl Guest for Component {
    async fn handle(url: String) -> Result<Body, ErrorCode> {
        host::sleep(10).await;
        let body = host::fetch(url).await?;
        let size = body.size().await;
        let (mut contents, trailers) = Body::consume(body);
        let mut bytes = Vec::new();
        loop {
            let (result, chunk) = contents.read(Vec::with_capacity(64)).await;
            bytes.extend(chunk);
            if let StreamResult::Dropped = result {
                break;
            }
        }
        let has_trailers = matches!(trailers.await, Ok(Some(_)));
        if bytes.is_empty() {
            host::report(ErrorContext::new("empty body"));
        }
        let (done_tx, done_rx) = wit_future::new(|| Ok(()));
        host::finish(done_rx);
        // The host may drop the future instead of reading it.
        let _ = done_tx.write(Ok(())).await;
        let reply = format!("size {size}, {} bytes, trailers {has_trailers}", bytes.len());
        Ok(Body::new(Some(write_stream(reply.into_bytes()))))
    }
    async fn serve(port: u16) -> StreamReader<u8> {
        write_stream(format!("serving on {port}").into_bytes())
    }
    fn echo(s: StreamReader<u8>) -> StreamReader<u8> {
        s
    }
}

/// A stream of `bytes`, which the host may stop reading at any point.
fn write_stream(bytes: Vec<u8>) -> StreamReader<u8> {
    let (mut tx, rx) = wit_stream::new();
    wit_bindgen::spawn_local(async move {
        let _ = tx.write_all(bytes).await;
    });
    rx
}

export!(Component);
