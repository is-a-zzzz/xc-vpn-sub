# vpn-sub-scraper

This is a simple web service that retrieves a VPN subscription link from `xcvpn.us`.

The service exposes a single endpoint `/` that, when accessed, logs into `xcvpn.us` using the provided credentials and returns the subscription link as a plain text response.

## How to Run

The application is containerized and can be run using Docker and Docker Compose.

### Prerequisites

- Docker
- Docker Compose

### Steps

1.  **Create an `.env` file:**

    Create a file named `.env` in the root of the project and add your `xcvpn.us` credentials:

    ```
    XCVPN_EMAIL=your_email@example.com
    XCVPN_PASSWORD=your_password
    ```

2.  **Build and run the service:**

    ```sh
    docker-compose up -d --build
    ```

3.  **Access the service:**

    The service will be running at `http://127.0.0.1:3000`. You can access the subscription link by visiting this URL in your browser or using a tool like `curl`:

    ```sh
    curl http://127.0.0.1:3000
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
    ```

3.  **Run the application:**

    ```sh
    ./target/release/vpn-sub-scraper
    ```

    The service will be available at `http://0.0.0.0:3000`.
