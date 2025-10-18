use std::{net::SocketAddr, sync::Arc};

use futures_channel::mpsc::unbounded;
use futures_util::{StreamExt, future, pin_mut};
use tokio::{
    net::TcpStream,
    sync::RwLock,
};
use tokio_tungstenite::WebSocketStream;

use crate::{
    error::Error, handler::handle_incoming_data, scene::Scene
};


pub struct WebSocketConnection {
    scene: Arc<RwLock<Scene>>,
    stream: WebSocketStream<TcpStream>,
}

impl WebSocketConnection {
    pub async fn new(
        scene: Arc<RwLock<Scene>>,
        raw_stream: TcpStream,
        address: SocketAddr,
    ) -> Result<Self, Error> {
        println!("WebSocket: New connection from {:?}", address);
        let stream = tokio_tungstenite::accept_async(raw_stream).await?;
        Ok(Self { scene, stream })
    }

    pub async fn run(self) {
        let (outgoing, mut incoming) = self.stream.split();
        let (tx, rx) = unbounded();
        let send_future = rx.map(Ok).forward(outgoing);
        let scene_clone = self.scene.clone();
        let receive_future = tokio::spawn(async move {
            while let Some(message) = incoming.next().await {
                let message = match message {
                    Ok(message) => message,
                    Err(error) => {
                        eprintln!("WebSocket: Error during message receive: {}", error);
                        return;
                    }
                };
                if !message.is_binary() && !message.is_text() {
                    return;
                }
                let data = message.into_data();

                match handle_incoming_data(self.scene.clone(), &data).await {
                    Err(error) => {
                        eprintln!("WebSocket: Received data format error: {}", error);
                        if let Err(send_error) =
                            tx.unbounded_send(format!("Error: {}", error).into())
                        {
                            eprintln!(
                                "WebSocket: Send error trying to tell client about previous data format error: {}",
                                send_error
                            );
                        }
                    }
                    Ok(false) => {
                        println!("WebSocket: Output end signaled");
                        break;
                    }
                    _ => {}
                }
            }
        });

        pin_mut!(send_future, receive_future);
        future::select(send_future, receive_future).await;

        println!("WebSocket: Disconnected");
        scene_clone.write().await.zero();
    }
}
