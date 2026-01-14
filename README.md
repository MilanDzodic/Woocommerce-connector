WooCommerce Retrieve Customer by ID

A connector project for retrieving a customer from WooCommerce based on the customer ID using the WooCommerce REST API.
The project includes an action that calls WooCommerce and RSpec tests written in Ruby to verify the functionality.


🚀 Functionality

This repository contains an action that:

✔️ Calls the WooCommerce REST API
✔️ Retrieves customer details by customer ID
✔️ Returns the customer object for further use in integrations

The WooCommerce customer endpoint used is:

GET /wp-json/wc/v3/customers/<id>

This endpoint returns a JSON object containing customer data (e.g. email, name, address) based on the provided ID.


🧱 Project Structure
├── src/                         # Action source code
├── spec/                        # Ruby tests (RSpec)
├── docker-compose.test.yml      # Docker-based test environment
├── .rspec                       # RSpec configuration
├── Gemfile / Gemfile.lock       # Ruby dependencies
├── Cargo.toml / build.rs        # Rust build configuration
└── README.md                    # Project documentation


🏁 Getting Started (Local)
🔧 Prerequisites

Make sure you have the following installed:

Docker & Docker Compose

Ruby (version defined in Gemfile)

Bundler (gem install bundler)

RSpec (bundle install)


🧪 Running the Test Environment

Start the test environment using Docker Compose:

docker compose -f docker-compose.test.yml up -d

If you encounter a container name conflict such as
/api-mock-server already in use, remove the existing container first:

docker rm -f api-mock-server


🧠 Ruby Tests (RSpec)

All tests are located in the spec/ directory and can be run with:

bundle exec rspec

The tests verify that:

✔️ The WooCommerce API request is built correctly
✔️ The response is handled as expected
✔️ Error scenarios are properly managed

🧩 Action Logic (Conceptual)

The action expects input similar to:

{
  "id": 123
}

The action performs the following steps:

Builds the WooCommerce API URL

Sends a GET request to /customers/<id>

Returns the customer object as JSON


📦 Example Usage

With a correctly configured connector client:

response = client.get("/customers/123")
puts response["email"]


📜 WooCommerce API Reference

WooCommerce REST API customer endpoint:

GET /wp-json/wc/v3/customers/<id>

This endpoint returns full customer details in JSON format.