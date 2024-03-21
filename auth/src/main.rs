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
    #[arg(short, env = "AUTH_API_PORT")]
    auth_api_port: u16,

    /// Auth database URL
    #[arg(env = "AUTH_POSTGRES_URL")]
    database_url: String,

    /// Web front-end url
    #[arg(env = "AUTH_WEB_FRONT_END_URL")]
    web_front_end_url: String,

    /// URL to connect with redis instance
    #[arg(env = "REDIS_SESSION_URL")]
    redis_session_storage_url: String,

    /// Private key for session storage
    #[arg(env = "PRIVATE_SESSION_KEY")]
    session_private_key: String,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();
    let args = Cli::parse();

    if let Err(e) = server::run_server(
        &args.grpc_port,
        &args.database_url,
        &args.web_front_end_url,
        &args.redis_session_storage_url,
        &args.session_private_key,
        args.auth_api_port,
    )
    .await
    {
        eprintln!("Error running auth microservice: {:?}", e);
        if let Some(source) = e.source() {
            eprintln!("error source: {:?}", source);
        }
    }
}
