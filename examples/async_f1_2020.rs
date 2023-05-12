use cm_telemetry::f1::f1_2020::F1_2020;
use cm_telemetry::AsyncTelemetryServer;

use std::time::Duration;

use tokio::time::{error::Elapsed, timeout};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let server = AsyncTelemetryServer::<F1_2020>::new("127.0.0.1:20777")
        .await
        .expect("failed to bind to address");
    println!("listening on 127.0.0.1:20777...");

    loop {
        let server_result: Result<Result<F1_2020, Box<dyn std::error::Error>>, Elapsed> =
            timeout(Duration::from_secs(10), server.next()).await;

        if let Err(timeout) = server_result {
            println!("Server timeout: {:?}", timeout);
            break; //Timeout occured
        }

        let event = server_result.unwrap(); //Get successful Result from server

        if let Err(e) = event {
            println!("error: {:?}", e);
            continue;
        }

        match event.unwrap() {
            F1_2020::Motion(data) => println!(
                "Motion packet received: {:?}",
                data.player_data().world_position
            ),
            F1_2020::Session(data) => println!(
                "Session packet received: {:?}, {:?}, {:?}, {:?}",
                data.formula, data.session_type, data.track, data.weather
            ),
            F1_2020::LapData(data) => println!(
                "LapData packet received: {:?}, {:?}, {:?}",
                data.player_data().current_lap_time,
                data.player_data().pit_status,
                data.player_data().driver_status,
            ),
            F1_2020::Event(data) => {
                println!("Event packet received: {:?}", data.event_data_details)
            }
            F1_2020::Participants(data) => println!(
                "Participants packet received: {:?}",
                data.player_data().name
            ),
            F1_2020::CarSetup(data) => println!(
                "CarSetups packet received: {:?}",
                data.player_data().fuel_load
            ),
            F1_2020::CarTelemetry(data) => println!(
                "CarTelemtry packet received: {:?}, {:?}",
                data.mfd_panel,
                data.player_data()
            ),
            F1_2020::CarStatus(data) => println!(
                "CarStatus packet received: {:?}",
                data.player_data().drs_activation_distance
            ),
            F1_2020::FinalClassification(data) => println!(
                "FinalClassification packet received: {:?}",
                data.player_data()
            ),
            F1_2020::LobbyInfo(data) => {
                println!("LobbyInfo packet received: {:?}", data.players(),)
            }
        }
    }
}
