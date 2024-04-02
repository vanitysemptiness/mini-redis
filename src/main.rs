use tokio::net::{ TcpListener, TcpStream };
use mini_redis::{ Connection, Frame };
use mini_redis::Command::{ self, Get, Set };
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    println!("Listening");

    /*
    Arc is for Atomic Reference Counted
    Arc makes data shareable concurrently betwen many tasks potentially running on many threads
    thread-safe: no race conditions or data races
    copy only increments the reference counter, cheap and efficient
    Mutex is a synchronization primative that only allows one thread access at a time
    race condition prevention
     */
    let db = new_shard_db(1);

    loop {
        // The second item contains the IP and port of the new connection.
        let (socket, _) = listener.accept().await.unwrap();

        let db = db.clone();
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        /*
        spawning the task subits it to the Tokio Scheduler
        this ensures the task gets executed when it has work to do
        spawned task may be executed on the same thread as where it was spawned
            or it may execute on a different runtime thread
        the task can be moved to different threads after being spawned
        a Task on requires one allocation and 64 bytes of memory
        apps should feel free to spawn millions of tasks if applicable
        A Task cannot have references to any values outside the task
        If a piece of data must be accessaible from more than one  task
            concurrently then it must be shared using sycnchronization primatives
            such as `Arc`
         */
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: DbShards) {
    // Connection, provided by `mini-redis`, handles parsing frames from
    // the socket
    let mut connection = Connection::new(socket);

    // Use `read_frame` to receive a command from the connection.
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                let mut db = db.lock().unwrap();
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };

        // Write the response to the client
        connection.write_frame(&response).await.unwrap();
    }
}

type DbShards = Arc<Vec<Mutex<HashMap<String, Vec<u8>>>>>;

/*
usize is an unsigned integer type that represents the size of a pointer on the
    current platform
    its used for indexes, sizes, and other quantities related to the memory
        layout of data structures
    4 bytes on a 32-bit system and 8 on a 64-bit system
*/
fn new_shard_db(num_shards: usize) -> DbShards {
    let mut db = Vec::with_capacity(num_shards);
    for _ in 0..num_shards {
        db.push(Mutex::new(HashMap::new()));
    }
    Arc::new(db) // putting a semi colon herre would not return the value
}

