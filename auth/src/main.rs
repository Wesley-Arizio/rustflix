use clap::Parser;

mod auth;
mod database;
mod grpc;
mod password_helper;
mod server;

#[derive(Parser, Debug)]
struct Cli {
    /// grpc port for Auth server
    #[arg(short, env = "AUTH_GRPC_PORT")]
    grpc_port: String,

    #[arg(env = "AUTH_MONGODB_URL")]
    mongodb_url: String,

    #[arg(env = "AUTH_DATABASE_NAME")]
    database_name: String,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();
    let args = Cli::parse();

    if let Err(e) =
        server::run_server(&args.grpc_port, &args.mongodb_url, &args.database_name).await
    {
        eprintln!("Error running auth microservice: {:?}", e);
        if let Some(source) = e.source() {
            eprintln!("error source: {:?}", source);
        }
    }
}
