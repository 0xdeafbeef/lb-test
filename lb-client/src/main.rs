use reqwest::Url;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let connect_addr = std::env::args().nth(1).expect("No arguments passed");
    let connect_addr: Url = connect_addr.parse().unwrap();
    let client = reqwest::ClientBuilder::default()
        .trust_dns(true)
        .build()
        .unwrap();
    let map = Arc::new(dashmap::DashMap::new());

    let sem = tokio::sync::Semaphore::new(100);
    let sem = Arc::new(sem);
    for i in 0..10_000_000 {
        let sem = sem.clone();
        let client = client.clone();
        let connect_addr = connect_addr.clone();
        let permit = sem.clone().acquire_owned().await.unwrap();
        let map = map.clone();

        tokio::spawn(async move {
            let res = client.get(connect_addr.clone()).send().await.unwrap();
            let ip = res
                .headers()
                .get("X-My-IP")
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();
            let _ = res.text().await;
            map.entry(ip).and_modify(|v| *v += 1).or_insert(1);
            drop(permit);
        });
    }

    loop {
        if sem.available_permits() == 100 {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    let mut map = Arc::into_inner(map).unwrap();

    for (k, v) in map.into_iter() {
        println!("{}: {}", k, v);
    }
}
