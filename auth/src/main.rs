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

    /// Auth API URL
    #[arg(short, env = "AUTH_API_ADDRESS")]
    auth_api_address: String,

    /// Auth database URL
    #[arg(env = "AUTH_POSTGRES_URL")]
    database_url: String,

    /// Web front-end url
    #[arg(env = "AUTH_WEB_FRONT_END_URL")]
    web_front_end_url: String,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();
    let args = Cli::parse();

    if let Err(e) = server::run_server(
        &args.grpc_port,
        &args.database_url,
        &args.auth_api_address,
        &args.web_front_end_url,
    )
    .await
    {
        eprintln!("Error running auth microservice: {:?}", e);
        if let Some(source) = e.source() {
            eprintln!("error source: {:?}", source);
        }
    }
}
