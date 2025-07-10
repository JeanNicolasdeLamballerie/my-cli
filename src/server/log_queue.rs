use once_cell::sync::OnceCell;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

pub static LOG_QUEUE: OnceCell<Arc<LogQueue>> = OnceCell::new();

#[derive(Debug)]
pub struct LogQueue {
    queue: Mutex<VecDeque<String>>,
    notify: Notify,
}

impl LogQueue {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            notify: Notify::new(),
        }
    }

    pub async fn push(&self, line: String) {
        let mut queue = self.queue.lock().await;
        queue.push_back(line);
        drop(queue);
        self.notify.notify_one();
    }

    pub async fn pop_all(&self) -> Vec<String> {
        let mut queue = self.queue.lock().await;
        queue.drain(..).collect()
    }

    pub async fn wait_for_log(&self) {
        self.notify.notified().await;
    }
}
pub async fn log_message(line: impl Into<String>) {
    if let Some(log_queue) = LOG_QUEUE.get() {
        log_queue.push(line.into()).await;
    }
}

pub fn init_log_buffer() -> Arc<LogQueue> {
    let log_queue = Arc::new(LogQueue::new());
    LOG_QUEUE
        .set(log_queue.clone())
        .expect("LOG_BUFFER already set");
    log_queue
}

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};
pub async fn handle_client(stream: &mut TcpStream) {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader).lines();

    let log_queue = LOG_QUEUE.get().expect("LOG_BUFFER not initialized").clone();

    loop {
        tokio::select! {
            maybe_line = reader.next_line() => {
                match maybe_line {
                    Ok(Some(line)) => {
                        if line.trim().is_empty() {
                            continue;
                        }
                        let response = match handle_command(line.clone()).await {
                            Ok(out) => out,
                            Err(e) => format!("Error: {e}")
                        };
                        if writer.write_all(response.as_bytes()).await.is_err() {
                            break;
                        }
                        let _ = writer.write_all(b"\n").await;
                    }
                    _ => break, // EOF or error
                }
            }

            _ = log_queue.notify.notified() => {
                let logs = log_queue.pop_all().await;
                for line in logs {
                    if writer.write_all(line.as_bytes()).await.is_err() {
                        return;
                    }
                    let _ = writer.write_all(b"\n").await;
                }
            }
        }
    }
}
async fn handle_command(line: impl Into<String>) -> anyhow::Result<String> {
    Ok(format!("line received : {}", line.into()))
}
