use clap::Parser;
use colored::Colorize;
use protocol::client::ClientToServer;
use protocol::common::Parse;
use protocol::server::ServerToClient;
use std::net::{SocketAddr, ToSocketAddrs};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, arg_required_else_help=false)]
struct Args {
    /// Set address to listen to
    #[arg(short, long, default_missing_value = "127.0.0.1:8080")]
    listen: String,

    /// Set address to target to
    #[arg(short, long, default_missing_value = "127.0.0.1:4242")]
    target: String,
}

fn print_diff_in_red(str1: &str, str2: &str) {
    let str1 = str1.replace("\n", "\\n").replace("\t", "\\t");
    let str2 = str2.replace("\n", "\\n").replace("\t", "\\t");
    for (char1, char2) in str1.chars().zip(str2.chars()) {
        if char1 != char2 {
            print!("{}", char1.to_string().red());
        } else {
            print!("{}", char1);
        }
    }

    println!(); // Ensure a newline at the end
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let listen_addr = args.listen;
    let target_addr = args.target;

    let listener = TcpListener::bind(&listen_addr)
        .await
        .expect("Failed to bind to address");

    println!(
        "{} {} {}",
        "[ Listening".magenta(),
        listen_addr,
        "]".magenta()
    );

    loop {
        let (client, client_addr) = listener.accept().await.expect("Failed to accept client");
        let target_addr = target_addr
            .to_socket_addrs()
            .unwrap()
            .next()
            .expect("Invalid target address");

        tokio::spawn(handle_client(client, target_addr));
        println!(
            "{} {} {}",
            "[ Accepted".bright_cyan(),
            client_addr,
            "]".bright_cyan()
        );
    }
}

async fn handle_client(mut client: TcpStream, target_addr: SocketAddr) {
    let mut target = TcpStream::connect(target_addr)
        .await
        .expect("Failed to connect to target");

    let (mut client_reader, mut client_writer) = client.split();
    let (mut target_reader, mut target_writer) = target.split();

    let client_to_target = copy_and_log(&mut client_reader, &mut target_writer, true);
    let target_to_client = copy_and_log(&mut target_reader, &mut client_writer, false);

    tokio::try_join!(client_to_target, target_to_client).expect("Failed to proxy data");
}

async fn copy_and_log<R, W>(
    reader: &mut R,
    writer: &mut W,
    client: bool,
) -> Result<u64, tokio::io::Error>
where
    R: AsyncReadExt + Unpin,
    W: AsyncWriteExt + Unpin,
{
    let mut buffer = [0u8; 2064];
    let mut total_bytes = 0;

    while let Ok(bytes_read) = reader.read(&mut buffer).await {
        if bytes_read == 0 {
            break;
        }

        let data = &buffer[..bytes_read];
        total_bytes += bytes_read as u64;

        let string = String::from_utf8(data.to_vec()).expect("failed to convert");

        for i in string.split("\n") {
            if i == "" {
                continue;
            }

            let arrow = match client {
                true => "=>".blue(),
                false => "<=".green(),
            };
            println!(
                "{} \"{}\\n\"",
                arrow,
                i.replace("\t", "\\t").replace("\n", "\\n").bold()
            );
            let i = i.to_owned() + "\n";
            if !client {
                let (input, cmd) = <ServerToClient>::parse(&i).expect("fucked");
                if !input.is_empty() {
                    println!("[[{}]]", input.replace("\t", "\\t").replace("\n", "\\n"));
                }

                if cmd.as_string() != i {
                    print_diff_in_red(&cmd.as_string(), &i);
                }
            } else {
                let (input, cmd) = <ClientToServer>::parse(&i).expect("fucked");
                if !input.is_empty() {
                    println!("[[{}]]", input.replace("\t", "\\t").replace("\n", "\\n"));
                }

                if cmd.as_string() != i {
                    print_diff_in_red(&cmd.as_string(), &i);
                }
            }
        }

        // Print the packet in hexadecimal format

        // Write the data to the target
        writer.write_all(data).await?;
    }

    Ok(total_bytes)
}
