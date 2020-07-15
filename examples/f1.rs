use cm_telemetry::f1::f1_2020::F1_2020;
use cm_telemetry::TelemetryServer;

fn main() {
    let server =
        TelemetryServer::<F1_2020>::new("127.0.0.1:20777").expect("failed to bind to address");

    loop {
        match server.next_event() {
            Ok(_) => println!("Got event packet :-)"),
            Err(e) => println!("Got an error :-(, {:?}", e),
        }
    }
}
