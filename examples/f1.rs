use cm_telemetry::f1::f1_2020::{Player, F1_2020};
use cm_telemetry::TelemetryServer;

fn main() {
    let server =
        TelemetryServer::<F1_2020>::new("127.0.0.1:20777").expect("failed to bind to address");
    println!("listening on 127.0.0.1:20777...");

    loop {
        let event = server.next();

        if let Err(e) = event {
            println!("error: {:?}", e);
            continue;
        }

        match event.unwrap() {
            F1_2020::Motion(_) => println!("Motion packet received"),
            F1_2020::Session(_) => println!("Session packet received"),
            F1_2020::LapData(_) => println!("LapData packet received"),
            F1_2020::Event(_) => println!("Event packet received"),
            F1_2020::Participants(_) => println!("Participants packet received"),
            F1_2020::CarSetups(_) => println!("CarSetups packet received"),
            F1_2020::CarTelemetry(_) => println!("CarTelemtry packet received"),
            F1_2020::CarStatus(_) => println!("CarStatus packet received"),
            F1_2020::FinalClassification(_) => println!("FinalClassification packet received"),
            F1_2020::LobbyInfo(_) => println!("LobbyInfo packet received"),
        }
    }
}
