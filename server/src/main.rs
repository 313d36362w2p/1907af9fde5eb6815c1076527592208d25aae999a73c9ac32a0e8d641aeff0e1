extern crate plugins;
extern crate utilities;

use utilities::plugins::{Plugin, PluginMessage, SystemMessage};

use std::process::exit;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixListener,
};

/// Stuff to fix thread safety across coroutines for remote closure of server.
use lazy_static::lazy_static;
use std::sync::Arc;
use tokio::sync::RwLock;

pub async fn start_server<T>(plugin: Arc<RwLock<T>>, path: &str) -> utilities::core::Result<()>
where
    T: Plugin + Clone,
    <T as Plugin>::SystemMessage: std::fmt::Display + Clone,
{
    // Remove socket if present at startup
    if let Err(err) = tokio::fs::remove_file(path).await {
        if err.kind() != std::io::ErrorKind::NotFound {
            return Err(err.into());
        }
    }

    // Bind the socket
    let listener = UnixListener::bind(path)?;
    println!("Server listening...");

    // Accept connections and spawn new tasks for multiple clients
    while let Ok((stream, _)) = listener.accept().await {
        // Clone the plugin (Arc) and pass it into the async task
        let plugin = Arc::clone(&plugin);

        tokio::spawn(async move {
            match handle_client(&plugin, stream).await {
                Ok(exit_flag) => {
                    if exit_flag {
                        exit(0);
                    }
                }
                Err(error) => {
                    println!("{:#?}", error);
                }
            }
        });
    }

    Ok(())
}

pub async fn handle_client<T>(
    plugin: &Arc<RwLock<T>>, // Use Arc<RwLock<T>> to get access to plugin
    stream: tokio::net::UnixStream,
) -> utilities::core::Result<bool>
where
    T: Plugin + Send, // Ensure T is Send to work across tasks
    <T as Plugin>::SystemMessage: std::fmt::Display + Clone,
{
    let mut stream = stream; // Keep the stream mutable
    let mut rx_bytes = Vec::new(); // Buffer to hold the incoming data

    // Read the buffer from client
    stream.read_to_end(&mut rx_bytes).await?;

    // Deserialize buffer into request message
    let rx_data = T::SystemMessage::deserialize(rx_bytes)?;

    // Lock the plugin to get mutable access (inside async block)
    let mut plugin = plugin.write().await; // `write()` to get a mutable reference

    // Call the handler
    let tx_data = T::pool_handler(&mut plugin, rx_data)?;

    // Quickly downcast the pool_handler's response
    let tx_message = utilities::core::downcast_system_message::<_, T::SystemMessage>(&tx_data)?;

    // Serialize the response message into a buffer
    let tx_bytes = T::SystemMessage::serialize(&tx_message)?;

    // Write the buffer to the stream
    stream.write_all(&tx_bytes).await?;
    stream.shutdown().await?;

    // Return the result of the shutdown check
    Ok(T::shutdown_check(&plugin)?)
}

/*
    Utility function to handle the networking runtime loop.
*/
async fn runtime<T>(plugin: Arc<RwLock<T>>) -> utilities::core::Result<()>
where
    T: Plugin,
    <T as Plugin>::PluginMessage: std::fmt::Display + Clone,
{
    // Bind to the UDP socket.
    let socket = tokio::net::UdpSocket::bind("127.0.0.1:9999").await?;

    // Prepare the buffer to receive the response.
    let mut buf = [0u8; 508];

    loop {
        // Receive data from the client
        let (len, client_addr) = socket.recv_from(&mut buf).await?;

        println!("Received {} bytes from {}", len, client_addr);

        // Return a vector of the response buffer as the client message.
        let client_message_bytes = (&buf[..len]).to_vec();

        // Downcast the client message bytes into the plugin message type, using the plugin message serializer.
        let client_message_downcast = T::PluginMessage::deserialize(client_message_bytes)?;

        // Lock the plugin to get mutable access (inside async block)
        let plugin = plugin.read().await; // `write()` to get a mutable reference

        // This is the main server plugin runtime call, it uses the plugin message and plugin state to determine execution flow.
        let server_message_generic = T::server_runtime(&plugin, client_message_downcast)?;

        // Downcast the response packet into the plugin message type, using the AsAny downcaster.
        let server_message_downcast = utilities::core::downcast_plugin_message::<
            _,
            T::PluginMessage,
        >(&server_message_generic)?;

        // Use the builtin plugin message serializer to self cast the message.
        let server_message_bytes = PluginMessage::serialize(server_message_downcast)?;

        // Send the message bytes over the UDP socket.
        socket.send_to(&server_message_bytes, &client_addr).await?;
    }
}

#[tokio::main]
async fn main() -> utilities::core::Result<()> {
    // Define GLOBAL_DATA using lazy_static
    lazy_static! {
        static ref GLOBAL_DATA: Arc<RwLock<plugins::MicrochipPlugin>> =
            Arc::new(RwLock::new(plugins::MicrochipPlugin {
                public_key: Vec::new(),
                private_key: Vec::new(),
                command_pool: Vec::new(),
                shutdown: false,
            }));
    }

    // Use GLOBAL_DATA to access the plugin across your application
    let plugin = GLOBAL_DATA.clone();

    // Assuming start_server is async, we await it here
    start_server(plugin.clone(), "tempestarii.socket").await?;

    // Assuming runtime expects the plugin, pass the cloned reference
    runtime::<plugins::MicrochipPlugin>(plugin.clone()).await?;

    Ok(())
}
