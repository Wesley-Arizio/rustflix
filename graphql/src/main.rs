use std::{io::Result, sync::Arc};

use actix_web::{
    route,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use juniper::http::GraphQLRequest;
use schemas::{create_schema, Schema};

pub mod query;
pub mod schemas;

#[route("/graphql", method = "POST")]
async fn graphql(schema: web::Data<Schema>, data: web::Json<GraphQLRequest>) -> impl Responder {
    let response = data.execute(&schema, &()).await;
    HttpResponse::Ok().json(response)
}

#[actix_web::main]
async fn main() -> Result<()> {
    let schema = Arc::new(create_schema());
    let app = move || {
        App::new()
            .app_data(Data::from(Arc::clone(&schema)))
            .service(graphql)
    };

    HttpServer::new(app).bind(("0.0.0.0", 8080))?.run().await?;

    Ok(())
}
