use cm_telemetry::dirt::rally2::DirtRally2;
use cm_telemetry::Server;

fn main() {
    let server = Server::<DirtRally2>::new().expect("failed to bind to address");

    loop {
        match server.next_event() {
            Ok(event) => println!(
                "Got event packet :-), {} m/s in {:?} gear!",
                event.car.speed, event.car.gear
            ),
            Err(e) => println!("Got an error :-(, {:?}", e),
        }
    }
}
