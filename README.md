WooCommerce Retrieve Customer by ID
A connector project for retrieving a customer from WooCommerce based on the customer ID using the WooCommerce REST API. The project consists of a Rust-based action compiled to WebAssembly (WASM) and verified with RSpec tests in Ruby.

🚀 Functionality
This action performs the following:

Calls the WooCommerce REST API: GET /wp-json/wc/v3/customers/<id>.

Handles dynamic path parameters for Customer IDs.

Configurable Error Handling: Supports strategies if a customer is not found (fail, continue, exit_level, exit_execution).

Returns the customer object as JSON for use in subsequent integration steps.

🧱 Project Structure
Plaintext

├── src/                         # Rust source code (Action & API Client)
├── spec/                        # Ruby tests (RSpec)
├── wit/                         # WebAssembly Interface definitions
├── target/                      # Compiled WASM (generated during build)
├── docker-compose.test.yml      # Docker-based test environment (Mock server)
├── Cargo.toml                   # Rust dependencies and configuration
└── Gemfile                      # Ruby dependencies for testing

🏁 Getting Started (Local)
🔧 Prerequisites
Rust with WASM target: rustup target add wasm32-wasip2

Docker & Docker Compose

Ruby & Bundler (gem install bundler)

🛠️ Development Workflow
To run the project and verify changes, follow these three steps:

1. Start the Test Environment (Mock Server)
We use a mock server to simulate the WooCommerce API without needing a live store.

Bash

docker compose -f docker-compose.test.yml up -d
If you encounter a container name conflict, run docker rm -f api-mock-server first.

2. Build the WASM Binary
The Rust code must be compiled into a WebAssembly component before the tests can be executed. This binary is what runs within the integration engine.

Bash

cargo build --release --target wasm32-wasip2
3. Run the Ruby Tests (RSpec)
Finally, run the tests which load the generated .wasm file and verify the logic against the mock server.

Bash

bundle install
bundle exec rspec
🧠 Tested Scenarios (RSpec)
The tests in spec/ verify:

✅ API requests are built with correct headers and URLs.

✅ Successful responses (200 OK) are parsed correctly from JSON.

✅ 404 Not Found handling based on the chosen strategy (e.g., returning null vs. raising an error).

✅ Input validation for both integer and string types for the Customer ID.

🧩 Action Logic
The action expects input following this schema:

JSON

{
  "customerId": 123,
  "on_not_found": "continue"
}
Extraction: The Rust code extracts customerId and builds the endpoint.

Request: A GET request is dispatched via the ApiClient.

Strategy: If the status code is 404, the on_not_found value determines if the execution should continue, stop the level, stop the execution, or fail.

Response: On 200 OK, the full customer JSON object is returned.

📜 WooCommerce API Reference
Standard endpoint for customers: GET /wp-json/wc/v3/customers/<id>

Would you like me to add a section on how to document the output schema so users know which fields (email, names, etc.) are available for mapping?
