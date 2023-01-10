use cm_telemetry::f1::f1_2022::F1_2022;
use cm_telemetry::TelemetryServer;

fn main() {
    let server =
        TelemetryServer::<F1_2022>::new("127.0.0.1:20777").expect("failed to bind to address");
    println!("listening on 127.0.0.1:20777...");

    loop {
        let event = server.next();

        if let Err(e) = event {
            println!("error: {:?}", e);
            continue;
        }

        match event.unwrap() {
            F1_2022::Motion(_data) => println!("Received Motion packet"),
            F1_2022::Session(_data) => println!("Received Session packet"),
            F1_2022::LapData(_data) => println!("Received Lap packet"),
            F1_2022::Event(_data) => println!("Received Event packet"),
            F1_2022::Participants(_data) => println!("Received Participants packet"),
            F1_2022::CarSetup(_data) => println!("Received CarSetup packet"),
            F1_2022::CarTelemetry(_data) => println!("Received CarTelemetry packet"),
            F1_2022::CarStatus(_data) => println!("Received CarStatus packet"),
            F1_2022::FinalClassification(_data) => println!("Received FinalClassification packet"),
            F1_2022::LobbyInfo(_data) => println!("Received LobbyInfo packet"),
            F1_2022::CarDamage(_data) => println!("Received CarDamage packet"),
            F1_2022::SessionHistory(_data) => println!("Received SessionHistory packet"),
        }
    }
}
