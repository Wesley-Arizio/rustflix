use clap::Parser;

mod auth;
mod grpc;
mod password_helper;
mod server;

#[derive(Parser, Debug)]
struct Cli {
    /// grpc port for Auth server
    #[arg(short, env = "AUTH_GRPC_PORT")]
    grpc_port: String,

    #[arg(env = "AUTH_POSTGRES_URL")]
    database_url: String,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();
    let args = Cli::parse();

    if let Err(e) = server::run_server(&args.grpc_port, &args.database_url).await {
        eprintln!("Error running auth microservice: {:?}", e);
        if let Some(source) = e.source() {
            eprintln!("error source: {:?}", source);
        }
    }
}
