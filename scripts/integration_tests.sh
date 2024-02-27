#!/bin/bash

if [ -f ../.env ]; then
    source ../.env
fi

# Define the path to your Docker Compose file
DOCKER_COMPOSE_FILE="../docker-compose.yaml"

# Check if the container is already running
if docker-compose -f "$DOCKER_COMPOSE_FILE" ps | grep -q "Up"; then
    echo "Container is already running. Stopping and removing the existing container..."

    # Stop and remove the existing container
    docker-compose -f "$DOCKER_COMPOSE_FILE" down
fi

# Start the container using Docker Compose
docker-compose -f "$DOCKER_COMPOSE_FILE" up -d

# Check if the container started successfully
if [ $? -eq 0 ]; then
    echo "Container started successfully."

    # wait a few seconds for the services to start
    sleep 10

    # run migrations in core database
    cd ../database/ && sqlx migrate run --database-url $SCRIPTS_CORE_DATABASE_URL

    # run e2e tests
    cd ../tests/ && k6 run ./create_credential.js -e GRAPHQL_CORE_URL=$GRAPHQL_CORE_URL
else
    echo "Failed to start the container."
fi
