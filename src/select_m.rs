use std::{collections::HashMap, time::Duration};

use tokio::{
    select,
    sync::{mpsc, oneshot},
    time::sleep,
};

async fn intro() {
    let (tx1, rx1) = oneshot::channel::<&str>();
    let (tx2, rx2) = oneshot::channel::<&str>();

    let _ = tx1.send("one");
    let _ = tx2.send("two");

    // the branch that not complete will droped
    select! {
        val = rx1 => {
            println!("Rcv msg: {val:?}");
        }
        val = rx2 => {
            println!("Rcv msg: {val:?}");
        }
    }
}

async fn operation() -> String {
    sleep(Duration::from_secs(1)).await;
    "Operation".to_string()
}
async fn cancellation() {
    let (mut tx1, rx1) = oneshot::channel::<&str>();
    let (tx2, rx2) = oneshot::channel::<&str>();

    tokio::spawn(async move {
        select! {
            val = operation() => {
                println!("{val}");
            }

            _ = tx1.closed() => {
                println!("TX1 Dropped");
            }


        }
    });

    tokio::spawn(async { tx2.send("TX2") });

    select! {
        val = rx1 => {
            println!("RX1 completes!!");
        }
        val = rx2 => {
            println!("RX2 completes!!");
        }
    }
}

async fn pattern_matching() {
    let (tx1, mut rx1) = mpsc::channel::<&str>(23);
    let (tx2, mut rx2) = mpsc::channel::<&str>(23);

    tokio::spawn(async move {
        // tx1.send("hey").await;
        tx1;
    });

    tokio::spawn(async move {
        tx2.send("hey").await;
        // tx2;
    });

    // if I drop one of the rx1/rx2 only one branch is invalidate since this will continue look for another to complete.
    // If both drops and resolve does not match with pattern it will  move to else block
    select! {
        Some(val) = rx1.recv() => {
            println!("Recv for RX1");
        }
        Some(val) = rx2.recv() => {
            println!("Recv for RX2");
        }
        else =>  {
            println!("Both dropeed");
        }
    }
}

async fn borrowing() {
    let (tx1, rx1) = oneshot::channel::<&str>();
    let (tx2, rx2) = oneshot::channel::<&str>();

    let mut map: HashMap<&str, String> = HashMap::new();

    map.insert("key", "data".to_string());

    let _ = tx1.send("key");
    let _ = tx2.send("not key");

    // two seprate futures can access the data map ref
    let map = &map;

    select! {
        Ok(key) = rx1 => {
            if let Some(dd) = map.get(key) {
                println!("Value is: {dd}");
            }
        }
        Ok(key) = rx2 => {
            if let Some(dd) = map.get(key) {
                println!("Value is: {dd}");
            } else {
                println!("Value is: None");
            }
        }
    }
}

async fn mut_borrowing() {
    let (tx1, rx1) = oneshot::channel::<&str>();
    let (tx2, rx2) = oneshot::channel::<&str>();

    let mut map: HashMap<&str, u32> = HashMap::new();

    map.insert("key", 1);

    let _ = tx1.send("key");
    let _ = tx2.send("not key");

    // two seprate futures can access the data map ref
    let map = &mut map;

    select! {
        Ok(key) = rx1 => {
            if let Some(v) = map.get(key) {
                println!("Old Value is: {v}");
                let vv = v.clone() + 1;
                map.insert(key, vv);
                println!("new Value is: {vv}");
            }
        }
        Ok(key) = rx2 => {
            if let Some(v) = map.get(key) {
                println!("Old Value is: {v}");
                let vv = v.clone() + 1;
                map.insert(key, vv);
                println!("new Value is: {vv}");
            } else {
                println!("Value is: None");
            }
        }
    }
}

pub async fn start() {
    // intro().await;
    // cancellation().await;
    // pattern_matching().await;
    borrowing().await;
    mut_borrowing().await;
}
