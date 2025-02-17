use rumqttd::{serialconfig::Config, Broker, Notification};
use std::{thread, time::Duration};

fn main() {
    pretty_env_logger::init();

    // As examples are compiled as seperate binary so this config is current path dependent. Run it
    // from root of this crate
    let config = config::Config::builder()
        .add_source(config::File::with_name("rumqttd.toml"))
        .build()
        .unwrap();

    let config: Config = config.try_deserialize().unwrap();

    dbg!(&config);

    let broker = Broker::new(config.into());
    let alerts = broker.alerts().unwrap();

    let (mut link_tx, mut link_rx) = broker.link("consumer").unwrap();
    link_tx.subscribe("hello/+/world").unwrap();
    thread::spawn(move || {
        let mut count = 0;
        loop {
            let notification = match link_rx.recv().unwrap() {
                Some(v) => v,
                None => continue,
            };

            match notification {
                Notification::Forward(forward) => {
                    count += 1;
                    println!(
                        "Topic = {:?}, Count = {}, Payload = {} bytes",
                        forward.publish.topic,
                        count,
                        forward.publish.payload.len()
                    );
                }
                v => {
                    println!("{v:?}");
                }
            }
        }
    });

    let handle = thread::spawn(move || loop {
        if let Ok(alert) = alerts.recv() {
            println!("Alert: {alert:?}");
        }
        thread::sleep(Duration::from_secs(1));
    });

    for i in 0..5 {
        let client_id = format!("client_{i}");
        let topic = format!("hello/{client_id}/world");
        let payload = vec![0u8; 1_000]; // 0u8 is one byte, so total ~1KB
        let (mut link_tx, _link_rx) = broker.link(&client_id).expect("New link should be made");

        thread::spawn(move || {
            for _ in 0..10 {
                thread::sleep(Duration::from_secs(1));
                link_tx.publish(topic.clone(), payload.clone()).unwrap();
            }
        });
    }

    handle.join().unwrap();
}
