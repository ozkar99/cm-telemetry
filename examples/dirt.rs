use cm_telemetry::dirt::rally2::DirtRally2;
use cm_telemetry::TelemetryServer;

fn main() {
    let server =
        TelemetryServer::<DirtRally2>::new("127.0.0.1:20777").expect("failed to bind to address");
    println!("listening on 127.0.0.1:20777...");

    loop {
        match server.next() {
            Ok(event) => println!(
                "Got event packet :-), {} m/s in {:?} gear!",
                event.car.speed, event.car.gear
            ),
            Err(e) => println!("Got an error :-(, {:?}", e),
        }
    }
}
