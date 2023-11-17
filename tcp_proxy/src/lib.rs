#![forbid(unsafe_code)]

use std::net::{TcpListener, TcpStream};
use std::thread;

use log::{error, info};

const LOCAL_HOST: &str = "127.0.0.1";

pub fn run_proxy(port: u32, destination: String) {
    let listener = TcpListener::bind(format!("{LOCAL_HOST}:{port}")).unwrap();

    info!("Proxy is listening on port: {}", port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let destination = destination.clone();
                thread::spawn(move || {
                    handle_connection(stream, &destination);
                });
            }
            Err(e) => {
                error!("Error accepting client connection: {e}");
            }
        }
    }
}

fn handle_connection(connection: TcpStream, destination: &str) {
    match TcpStream::connect(destination) {
        Ok(server_stream) => {
            info!("Connected to destination: {destination}");

            let (mut client_reader, mut client_writer) = (
                connection.try_clone().unwrap(),
                connection.try_clone().unwrap(),
            );
            let (mut server_reader, mut server_writer) = (
                server_stream.try_clone().unwrap(),
                server_stream.try_clone().unwrap(),
            );

            let client_to_server = thread::spawn(move || {
                std::io::copy(&mut client_reader, &mut server_writer).unwrap();
                info!("Client -> server");
            });

            let server_to_client = thread::spawn(move || {
                std::io::copy(&mut server_reader, &mut client_writer).unwrap();
                info!("Server -> client");
            });

            client_to_server.join().unwrap();
            server_to_client.join().unwrap();

            connection
                .shutdown(std::net::Shutdown::Both)
                .expect("shutdown call failed");
            info!("Client stream stutted down");
            server_stream
                .shutdown(std::net::Shutdown::Both)
                .expect("shutdown call failed");
            info!("Server stream stutted down");
        }
        Err(e) => error!("Error connecting to destination: {e}"),
    }
}
