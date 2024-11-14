extern crate plugins;
extern crate utilities;

use utilities::plugins::{Plugin, PluginMessage};

/*
    Utility function to handle the networking runtime loop.
*/
async fn runtime<T>(plugin: T) -> utilities::core::Result<()> where T: Plugin, <T as Plugin>::PluginMessage: std::fmt::Display + Clone {
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

        // This is the main server plugin runtime call, it uses the plugin message and plugin state to determine execution flow.
        let server_message_generic = T::server_runtime(&plugin, client_message_downcast)?;
        
        // Downcast the response packet into the plugin message type, using the AsAny downcaster.
        let server_message_downcast = utilities::core::downcast_message::<_, T::PluginMessage>(&server_message_generic)?;

        // Use the builtin plugin message serializer to self cast the message.
        let server_message_bytes = PluginMessage::serialize(server_message_downcast)?;
        
        // Send the message bytes over the UDP socket.
        socket.send_to(&server_message_bytes, &client_addr).await?;

    }
}


#[tokio::main]
async fn main() -> utilities::core::Result<()> {
    let plugin = plugins::MicrochipPlugin {
        public_key: Vec::new(),
        private_key: Vec::new(),
    };


    runtime::<plugins::MicrochipPlugin>(plugin).await?;
    
    Ok(())
}
