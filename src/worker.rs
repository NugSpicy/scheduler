/*
Supervisor
    Establish DB Connection
    Get Tasks (BATCH_SIZE and start_ts <= NOW())
    Spawn Worker threads to handle each task
    Maintain Vec<Worker>
    Vec size CONCURRENT_MAX (Should be lower than BATCH_SIZE)
    If worker_count < CONCURRENT_MAX - BATCH_SIZE
        Get Tasks

Worker
    Route Task to fn()
    Run fn()
    Mark Task as COMPLETE
*/

use anyhow::Result;
use hyper::{Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use rand::Rng;
use std::time::Duration;
use tokio::{
    sync::mpsc::{channel, Sender},
    task::{spawn, JoinHandle},
};
use uuid::Uuid;

use crate::{
    adapters::scylla::ScyllaConnection,
    models::{complete_task, process_tasks, Task},
};

static CONCURRENT_MAX: i32 = 250;
static BATCH_SIZE: i32 = 50;

#[derive(Debug)]
pub struct WorkerError {}

pub struct Supervisor {
    _id: Uuid,
    _thread: JoinHandle<()>,
}

impl Supervisor {
    pub fn new(db: ScyllaConnection) -> Result<Self> {
        let id = Uuid::new_v4();
        let thread = spawn(async move {
            supervise(db, &id).await;
        });

        Ok(Self {
            _id: id,
            _thread: thread,
        })
    }
}

pub async fn supervise(db: ScyllaConnection, id: &Uuid) {
    let mut count = 0;
    let (sender, mut receiver) = channel(256);
    loop {
        while let Ok(_msg) = receiver.try_recv() {
            count -= 1;
        }

        if count <= CONCURRENT_MAX - BATCH_SIZE {
            let session = db.session().await.unwrap();
            // Get Tasks
            let tasks: Vec<Task> = process_tasks(session, id, BATCH_SIZE).await.unwrap();

            // Spawn tokio::Task for each Task w/ Send channel
            for task in tasks {
                spawn(handle_task(sender.clone(), task, (&db).clone()));
                count += 1;
            }
        } else {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

async fn handle_task(sender: Sender<Result<()>>, task: Task, db: ScyllaConnection) {
    sender.send(task_match(task, db).await).await.unwrap();
}

async fn task_match(task: Task, db: ScyllaConnection) -> Result<()> {
    match task.task_type.as_str() {
        "foo" => {
            tokio::time::sleep(Duration::from_millis(3000)).await;
            println!("Foo {}", task.id);
        }
        "bar" => {
            let https = HttpsConnector::new();
            let client = Client::builder().build::<_, hyper::Body>(https);

            let req = Request::builder()
                .method(Method::GET)
                .header("User-Agent", "Chrome/113.0.0.0")
                .header("Accept", "text/html")
                .uri("https://www.whattimeisitrightnow.com")
                .body(Body::from(""))?;
            let resp = client.request(req).await?;
            println!("{}", resp.status());
        }
        "baz" => {
            let mut rng = rand::thread_rng();

            println!("Baz {}", rng.gen_range(0..344));
        }
        _ => println!("Erroneus task found."),
    }

    let sess = db.session().await?;
    complete_task(sess, task).await.unwrap();

    Ok(())
}
