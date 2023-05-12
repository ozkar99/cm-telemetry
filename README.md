# cm-telemetry

Implements the codemasters (and friends) UDP telemetry protocol
and provides an abstraction for interacting with it.

## Supported Games

- F1 2020
- F1 2022
- Dirt Rally 2.0

## Example

```rust
use cm_telemetry::f1::f1_2020::F1_2020;
use cm_telemetry::TelemetryServer;

fn main() {
    let server =
        TelemetryServer::<F1_2020>::new("127.0.0.1:20777").expect("failed to bind to address");

    loop {
        let event = server.next();

        if let Err(e) = event {
            println!("error: {:?}", e);
            continue;
        }

        match event.unwrap() {
            F1_2020::FinalClassification(_data) => println!("Received FinalClassification packet"),
            F1_2020::LobbyInfo(_data) => println!("Received LobbyInfo packet"),
            _ => ()
        }
```

### Using Externally Defined Games

You can support new games (external to the crate) by implementing the `Event` trait on a type
and then initialize a `Server` with it using `cm_telemetry::Server::<T>::new(addr)`

PR's welcome :)

### Furter Reading

[F1 2020 UDP Specification](https://web.archive.org/web/20221127112921/https://forums.codemasters.com/topic/50942-f1-2020-udp-specification/)       (Webarchive because EA)

[F1 2022 UDP Specification](https://answers.ea.com/t5/General-Discussion/F1-22-UDP-Specification/td-p/11551274)
