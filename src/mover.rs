use std::time::Instant;

use crate::{database, tcp_log};

use diesel::SqliteConnection;
use log::trace;
use tokio::net::TcpStream;
//use std::io;
pub async fn move_to(name: &str, conn: &mut SqliteConnection, stream: &mut TcpStream) {
    let timestamp_start = Instant::now();

    trace!("Requesting db access :");
    let timestamp_end = Instant::now();
    // let mut conn = establish_connection();
    let duration = timestamp_end.duration_since(timestamp_start);

    trace!(
        "Db access granted in : {} ms, moving to requesting db...",
        duration.as_millis()
    );

    let timestamp_start = Instant::now();
    let project = database::fetch_single_project(conn, name);

    let timestamp_end = Instant::now();
    let duration = timestamp_end.duration_since(timestamp_start);

    trace!(
        "Db request granted in : {} ms, printing result.",
        duration.as_millis()
    );
    tcp_log!(stream => "{}", project.path).await;
}
