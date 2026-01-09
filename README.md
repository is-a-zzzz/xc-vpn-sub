# vpn-sub-scraper

This is a simple web service that retrieves a VPN subscription from `xcvpn.us`.

The service exposes two endpoints:
- `/` - Returns the subscription link as a plain text response
- `/res` - Returns detailed subscription information as JSON

## Project Structure

The project is organized into the following modules:
- `main.rs` - Application entry point and server setup
- `config.rs` - Configuration management from environment variables
- `client.rs` - HTTP client for making API requests
- `service.rs` - Business logic for VPN subscription retrieval
- `handlers.rs` - HTTP request handlers
- `error.rs` - Error types and handling

## How to Run

The application is containerized and can be run using Docker and Docker Compose.

### Prerequisites

- Docker
- Docker Compose

### Steps

1.  **Create an `.env` file:**

    Create a file named `.env` in the root of the project and add your `xcvpn.us` credentials:

    ```env
    XCVPN_EMAIL=your_email@example.com
    XCVPN_PASSWORD=your_password
    SERVER_HOST=0.0.0.0
    SERVER_PORT=3000
    LOGIN_URL=https://xcvpn.us/api/v1/passport/auth/login
    SUBSCRIBE_URL=https://xcvpn.us/api/v1/user/getSubscribe
    ```

    **Note:** `SERVER_HOST`, `SERVER_PORT`, `LOGIN_URL`, and `SUBSCRIBE_URL` are optional and have default values.

2.  **Build and run the service:**

    ```sh
    docker-compose up -d --build
    ```

3.  **Access the service:**

    The service will be running at `http://127.0.0.1:3000`.

    - Get subscription link (plain text):
      ```sh
      curl http://127.0.0.1:3000
      ```

    - Get detailed subscription info (JSON):
      ```sh
      curl http://127.0.0.1:3000/res
      ```

## Building from source (without Docker)

You can also build and run the project directly using Cargo.

### Prerequisites

- Rust toolchain

### Steps

1.  **Install dependencies and build:**

    ```sh
    cargo build --release
    ```

2.  **Set environment variables:**

    ```sh
    export XCVPN_EMAIL=your_email@example.com
    export XCVPN_PASSWORD=your_password
    export SERVER_HOST=0.0.0.0
    export SERVER_PORT=3000
    export LOGIN_URL=https://xcvpn.us/api/v1/passport/auth/login
    export SUBSCRIBE_URL=https://xcvpn.us/api/v1/user/getSubscribe
    ```

    **Note:** Only `XCVPN_EMAIL` and `XCVPN_PASSWORD` are required. Other variables have default values.

3.  **Run the application:**

    ```sh
    ./target/release/vpn-sub-scraper
    ```

    The service will be available at `http://0.0.0.0:3000`.
