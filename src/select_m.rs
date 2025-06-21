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
        _val = rx1 => {
            println!("RX1 completes!!");
        }
        _val = rx2 => {
            println!("RX2 completes!!");
        }
    }
}

async fn pattern_matching() {
    let (tx1, mut rx1) = mpsc::channel::<&str>(23);
    let (tx2, mut rx2) = mpsc::channel::<&str>(23);

    tokio::spawn(async move {
        // tx1.send("hey").await;
        drop(tx1);
    });

    tokio::spawn(async move {
        tx2.send("hey").await.unwrap();
        // tx2;
    });

    // if I drop one of the rx1/rx2 only one branch is invalidate since this will continue look for another to complete.
    // If both drops and resolve does not match with pattern it will  move to else block
    select! {
        Some(_val) = rx1.recv() => {
            println!("Recv for RX1");
        }
        Some(_val) = rx2.recv() => {
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

pub async fn loops() {
    let (tx1, mut rx1) = mpsc::channel::<&str>(20);
    let (tx2, mut rx2) = mpsc::channel::<&str>(20);

    tokio::spawn(async move {
        tx1.send("MSG1").await.unwrap();
    });

    tokio::spawn(async move {
        tx2.send("MSG2").await.unwrap();
    });

    loop {
        select! {
            Some(msg) = rx1.recv() => {
                println!("msg comes: {msg}");
            }
            Some(msg) = rx2.recv() => {
                println!("msg comes: {msg}");
            }
            else => {
                println!("All done");
                break;
            }
        }
    }

    println!("Out of loop: All of the msg got recived");
}

pub async fn start() {
    println!(
        "
           A verry simple intro of select!.
           One shot channel only one would recive one will drop.
        "
    );
    intro().await;

    println!(
        "
           A verry simple cancellation in select!.
           Two spawn task got created that will either complete or drop if another task got completed first. 
           One shot channel only one would recive one will drop.
        "
    );
    cancellation().await;

    println!(
        "
           A verry simple pattern matching in select!.
           If onw will not match go to next one and wait for it if any complete drop others.
           If no one match excute else.
        "
    );
    pattern_matching().await;

    println!(
        "
           Both branches of select! take a ref of same data.        
        "
    );
    borrowing().await;

    println!(" Both branches of select! take a mut ref of same data.");
    mut_borrowing().await;

    println!(
        "
           A verry simple into of select!.
        "
    );
    loops().await;
}
