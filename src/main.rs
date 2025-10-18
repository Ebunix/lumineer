use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{net::ToSocketAddrs, path, process::exit, sync::Arc};

use artnet_protocol::{ArtCommand, Poll};
use clap::Parser;
use futures_util::future;
use rouille::Response;
use tokio::{
    net::{TcpListener, UdpSocket},
    sync::RwLock,
};

use crate::handler::handle_incoming_data;
use crate::{
    config::{Args, Config},
    connection::WebSocketConnection,
    error::Error,
    scene::Scene,
};

mod config;
mod connection;
mod dmx;
mod error;
mod feature;
mod fixture;
mod handler;
mod scene;
mod universe;

const DEFAULT_ARTNET_PORT: u16 = 6454;

async fn run(root: PathBuf, config: Config) -> Result<(), Error> {
    let scene = Arc::new(RwLock::new(Scene::from_config(&config.scene)?));
    let passcode = config.lumineer.passcode.clone();

    // Setup frontend listener
    let frontend_local_address = config.lumineer.local_address.clone();
    let frontend_passcode = passcode.clone();
    let frontend_future = tokio::spawn(async move {
        let frontend_address = (
            frontend_local_address.clone(),
            config.lumineer.frontend_port,
        )
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap();
        let frontend_root = root.join(Path::new(&config.lumineer.frontend_root));
        println!(
            "HTTP: Frontend available at http://{}?{}",
            frontend_address, &frontend_passcode
        );
        rouille::start_server(frontend_address, move |request| {
            let mut response = rouille::match_assets(request, &frontend_root);
            if response.is_success() {
                response = response.with_additional_header(
                    "X-Websocket-Address",
                    format!(
                        "ws://{}:{}",
                        frontend_local_address, config.lumineer.websocket_port
                    ),
                );
                return response;
            }
            let path = frontend_root.join("index.html");
            if let Ok(file) = File::open(path) {
                return Response::from_file("text/html", file);
            }
            Response::empty_404()
        });
    });

    // Setup websocket listener
    let websocket_scene = scene.clone();
    let websocket_passcode = passcode.clone();
    let websocket_local_address = config.lumineer.local_address.clone();
    let websocket_future = tokio::spawn(async move {
        let websocket_address = (websocket_local_address, config.lumineer.websocket_port)
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap();
        let websocket_listener = TcpListener::bind(websocket_address)
            .await
            .expect("WebSocket: Failed to bind");
        println!("WebSocket: Listening on {}", websocket_address);
        while let Ok((stream, address)) = websocket_listener.accept().await {
            let websocket_scene = websocket_scene.clone();
            let passcode = websocket_passcode.clone();
            tokio::spawn(async move {
                match WebSocketConnection::new(websocket_scene.clone(), stream, address, &passcode)
                    .await
                {
                    Ok(connection) => {
                        connection.run().await;
                    }
                    Err(error) => {
                        eprintln!("WebSocket: connection failed {error}");
                    }
                };
            });
        }
    });

    // Setup UDP listener
    let udp_scene = scene.clone();
    let udp_local_address = config.lumineer.local_address.clone();
    let udp_future = tokio::spawn(async move {
        let mut recv_buffer = [0u8; 4096];

        let udp_address = (udp_local_address, config.lumineer.udp_port)
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap();
        let udp_socket = UdpSocket::bind(udp_address)
            .await
            .expect("UDP: failed to bind");
        println!("UDP: Listening on {}", udp_address);
        loop {
            match udp_socket.recv_from(&mut recv_buffer).await {
                Ok(_) => {
                    match handle_incoming_data(udp_scene.clone(), &recv_buffer, &passcode).await {
                        Err(error) => {
                            eprintln!("UDP: Received data format error: {}", error);
                            //if let Err(send_error) = tx.unbounded_send(format!("Error: {}", error).into()) {
                            //    eprintln!(
                            //        "WebSocket: Send error trying to tell client about previous data format error: {}",
                            //        send_error
                            //    );
                            //}
                        }
                        Ok(false) => {
                            println!("UPD: Output end signaled");
                            udp_scene.write().await.zero();
                        }
                        _ => {}
                    }
                }
                Err(error) => {
                    eprintln!("UDP: Receive error {error}");
                }
            }
        }
    });

    // Create ArtNet local address to use for incoming messages
    let local_artnet_address = ("0.0.0.0", DEFAULT_ARTNET_PORT)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();
    let local_artnet_socket = Arc::new(UdpSocket::bind(local_artnet_address).await?);

    // Remote ArtNet address used by the target node that receives our frames
    let remote_artnet_address = (config.lumineer.artnet_remote, DEFAULT_ARTNET_PORT)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

    // Setup ArtNet listener to poll for available devices. This is
    // not required, but nice to have for debugging purposes to see what
    // devices on the network respond to ArtNet
    let artnet_receive_socket = local_artnet_socket.clone();
    let artnet_receive_future = tokio::spawn(async move {
        let poll_buffer = ArtCommand::Poll(Poll::default()).write_to_buffer().unwrap();
        artnet_receive_socket
            .send_to(&poll_buffer, &remote_artnet_address)
            .await
            .unwrap();

        println!("ArtNet: Input bound on address {:?}", local_artnet_address);

        loop {
            let mut buffer = [0u8; 1024];
            let (length, _addr) = artnet_receive_socket.recv_from(&mut buffer).await.unwrap();
            let command = ArtCommand::from_buffer(&buffer[..length]).unwrap();

            match command {
                ArtCommand::PollReply(reply) => {
                    let long_name = String::from_utf8(reply.long_name.to_vec())
                        .unwrap_or("(Invalid name)".to_owned());
                    let short_name =
                        String::from_utf8(reply.short_name.to_vec()).unwrap_or("(???)".to_owned());
                    println!(
                        "ArtNet: Device \"{long_name} {short_name}\" at {:?} ",
                        reply.address
                    );
                }
                _ => {}
            }
        }
    });

    // Yay, the actual output loop! Currently this sends DMX data every 10ms, which is
    // not really close to what the spec wants, but most (if not all) nodes seem to handle
    // this just fine, so I'll keep it until it breaks.
    let dmx_out_socket = local_artnet_socket.clone();
    let dmx_out_scene = scene.clone();
    let dmx_out_future = tokio::spawn(async move {
        println!(
            "ArtNet: Output bound on address {:?}",
            remote_artnet_address
        );

        loop {
            let lock = dmx_out_scene.read().await;
            for output in lock.iter_output() {
                let command = ArtCommand::Output(output);
                let buffer = command.write_to_buffer().unwrap();
                if let Err(error) = dmx_out_socket.send_to(&buffer, remote_artnet_address).await {
                    eprintln!("ArtNet: Send error {}", error);
                }
            }
            drop(lock);
            tokio::time::sleep(Duration::from_millis(1000 / 40)).await;
        }
    });

    // Await all the futures we created above, keeps the program from quitting immediately
    // which is kinda cool if it's supposed to keep running ;)
    future::select_all([
        websocket_future,
        udp_future,
        artnet_receive_future,
        dmx_out_future,
        frontend_future,
    ])
    .await
    .0?;

    Ok(())
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let full_config_path = path::absolute(&args.config).unwrap_or_default();
    let config_file_data = fs::read(&full_config_path).unwrap_or_default();
    let config = match toml::from_slice::<Config>(&config_file_data) {
        Ok(data) => data,
        Err(error) => {
            eprintln!(
                "Invalid configuration file at {:?}: {}",
                full_config_path, error
            );
            exit(-1);
        }
    };
    let root = PathBuf::from(full_config_path.parent().unwrap());
    if let Err(error) = run(root, config).await {
        eprintln!("Error: {error}");
    }
}
