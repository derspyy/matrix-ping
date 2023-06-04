use anyhow::Error;
use async_std::{net::ToSocketAddrs, task};
use futures::future::join_all;
use serde::Deserialize;
use surge_ping::ping as surge_ping;

#[derive(Deserialize)]
struct List {
    domain: String,
}

fn main() {
    let list: Vec<List> = ureq::get("https://joinmatrix.org/servers.json")
        .call()
        .unwrap()
        .into_json()
        .unwrap();
    let mut tasks = Vec::new();
    let mut ping_list = Vec::new();
    for x in list {
        let task = task::spawn(ping(x.domain));
        tasks.push(task)
    }

    for x in task::block_on(join_all(tasks)) {
        match x {
            Ok(result) => ping_list.push(result),
            Err(e) => eprintln!("\x1B[31m{e}"),
        }
    }
    ping_list.sort();
    let mut count = 1;
    for server in ping_list {
        println!("\x1B[35m{count}.\x1B[0m {}: {}ms", server.1, server.0);
        count += 1;
    }
}

async fn ping(domain: String) -> Result<(u128, String), Error> {
    let new_domain = if domain.find(':').is_none() {
        format!("{domain}:8080")
    } else {
        domain.clone()
    };

    let addr = new_domain.to_socket_addrs().await?.next().unwrap();

    let ping = surge_ping(addr.ip(), &[1, 2, 3, 4])
        .await
        .map_err(|_| anyhow::anyhow!("timed out for: {domain}"))?;
    Ok((ping.1.as_millis(), domain))
}
