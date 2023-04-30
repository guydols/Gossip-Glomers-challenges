pub mod helper;
pub mod protocol;
pub mod node;

use protocol::Message;
use std::time::Duration;
use node::Node;
use async_channel::{unbounded, Receiver, Sender};
use async_std::io::{stdin, stdout, WriteExt};
use async_std::sync::{Arc, Mutex};
use tokio::time::interval;

// Time interval in millis
const GOSSIP_INTERVAL: usize = 900;

#[tokio::main]
async fn main() {
    let node = Arc::new(Mutex::new(Node::new()));

    let (reader_tx, reader_rx) = unbounded::<Message>();
    let (writer_tx, writer_rx) = unbounded::<Message>();

    let t1 = tokio::spawn(stdin_reader(reader_tx.clone()));
    let t2 = tokio::spawn(io_handler(
        node.clone(),
        reader_rx.clone(),
        writer_tx.clone(),
    ));
    let t3 = tokio::spawn(stdout_writer(writer_rx.clone()));
    let t4 = tokio::spawn(gossip_handler(node.clone(), writer_tx.clone()));

    tokio::join!(t1, t2, t3, t4);
}

async fn stdin_reader(tx: Sender<Message>) {
    let stdin = stdin();
    loop {
        let mut buffer = String::new();
        stdin.read_line(&mut buffer).await.unwrap();
        let new_msg = serde_json::from_str::<Message>(&buffer).unwrap();
        tx.send(new_msg.clone()).await.unwrap();
    }
}

async fn io_handler(node: Arc<Mutex<Node>>, rx: Receiver<Message>, tx: Sender<Message>) {
    loop {
        let msg = rx.recv().await.unwrap();
        // Todo: Unregister msg if reply was anticipated
        let reply = match msg.handle(&mut *node.lock_arc().await) {
            Ok(msg) => msg,
            Err(_) => todo!(),
        };
        // Todo: Register reply if reply needed
        if reply.is_some() {
            tx.send(reply.unwrap().clone()).await.unwrap();
        }
    }
}

async fn gossip_handler(node: Arc<Mutex<Node>>, tx: Sender<Message>) {
    loop {
        let mut interval = interval(Duration::from_millis(GOSSIP_INTERVAL.try_into().unwrap()));
        interval.tick().await;
        interval.tick().await;
        let borrow_node = &mut *node.lock_arc().await;
        let messages = match borrow_node.talk() {
            Some(msgs) => msgs,
            None => vec![],
        };
        // Todo: Register reply if reply needed
        if !messages.is_empty() {
            for msg in messages {
                tx.send(msg.clone()).await.unwrap();
            }
        }
    }
}

async fn stdout_writer(rx: Receiver<Message>) {
    let mut stdout = stdout();
    loop {
        let msg = rx.recv().await.unwrap();
        let mut buf = serde_json::to_vec(&msg).unwrap();
        buf.push(10);
        stdout.write_all(&buf).await.unwrap();
        stdout.flush().await.unwrap();
    }
}
