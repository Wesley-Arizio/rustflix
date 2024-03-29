## Rustflix

This project is a movie catalog API, built to learn how to work with microservices in rust.
On top of that, it also counts with a cookie-based authentication to ensure security.

The project contain unit and integration tests to run them:
```shell
cargo t # unit tests
cd scripts && ./integration_tests.sh # integration tests, to run you start docker on your machine
```

### Technologies
 
 - Rust
 - Sqlx (Postgres)
 - Actix-web & juniper for graphql api
 - Docker
 - gRPC
 - Redis for session storage

### How's the project structured?

   - **Database**
   There's two databases for this project, one handle the auth microservice and the other one handles the core business rules.
   Both of them follow the same structure and trait, so it's easy to add a new one if needed.
   - **Auth** microservice handles the authentication and credential storage, it exposes a gRPC server that is used to create credentials and
   validate their access.
   - **Core** is a library crate that exposes a few methods that wraps the core business rules implementation, you can use it in a graphql or a rest api client.
   - **Graphql-core** is a web server that exposes a graphql api to access a few methods of our core business rules like list movies.
   - **Grpc-interfaces** is a library that exposes a few interfaces like auth client and server.