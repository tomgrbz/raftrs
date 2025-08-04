use clap::Parser;
use raft_rs::{ConnInfo, ConnectionGroup, Replica, Ticks};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let conn_info = ConnInfo::new(cli.port)
        .await
        .expect("Failed to bind to socket");
    let conn_group = ConnectionGroup::new(conn_info);
    let mut rep = Replica::new(cli.id, cli.others, conn_group).await.unwrap();

    rep.run().await.unwrap();
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    port: u16,
    id: String,
    others: Vec<String>,
}
