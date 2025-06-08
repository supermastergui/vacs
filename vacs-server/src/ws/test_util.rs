use axum::extract::ws;
use futures_util::{Sink, Stream};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;

pub struct MockSink {
    tx: mpsc::UnboundedSender<ws::Message>,
}

impl MockSink {
    pub fn new(tx: mpsc::UnboundedSender<ws::Message>) -> Self {
        Self { tx }
    }
}

impl Sink<ws::Message> for MockSink {
    type Error = axum::Error;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: ws::Message) -> Result<(), Self::Error> {
        self.tx.send(item).map_err(|e| axum::Error::new(e))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

pub struct MockStream {
    messages: Vec<Result<ws::Message, axum::Error>>,
}

impl MockStream {
    pub fn new(messages: Vec<Result<ws::Message, axum::Error>>) -> Self {
        Self { messages }
    }
}

impl Stream for MockStream {
    type Item = Result<ws::Message, axum::Error>;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.messages.is_empty() {
            Poll::Ready(None)
        } else {
            Poll::Ready(Some(self.messages.remove(0)))
        }
    }
}
