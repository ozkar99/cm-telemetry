use cm_telemetry;

fn main() {
    let dirt = cm_telemetry::dirt::rally2::DirtRally2::new().expect("failed to bind to address");

    loop {
        match dirt.next_event() {
            Ok(event) => println!(
                "Got event packet :-), {} m/s in {:?} gear!",
                event.car.speed, event.car.gear
            ),
            Err(e) => println!("Got an error :-(, {:?}", e),
        }
    }
}
