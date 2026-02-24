require 'spec_helper'
require 'json'

# Mock implementation of the action logic
module UpdateCustomerAction
  def self.execute(context)
    begin
      connection_params = JSON.parse(context.connection.serialized_data)
    rescue JSON::ParserError => e
      raise "Invalid connection configuration: #{e.message}"
    end

    input = JSON.parse(context.serialized_input)

    # Extract customerId for the URL
    customer_id = input["customerId"]
    raise "customerId parameter is required" if customer_id.nil?

    # Matching Rust logic: remove empty values AND the customerId from the body
    request_body = input.reject { |k, v| v == "" || k == "customerId" }

    client = ApiClient.new(connection_params)

    endpoint = "/customers/#{customer_id}"
    status, body = client.put(endpoint, request_body)

    raise "WooCommerce returned error code #{status}: #{body}" if status >= 400

    begin
      JSON.parse(body)
    rescue JSON::ParserError => e
      raise "Failed to parse JSON response: #{e.message}"
    end
  end
end

RSpec.describe "UpdateCustomerAction" do
  let(:api_client) { double('ApiClient') }
  let(:customer_id) { "123" }
  let(:endpoint) { "/customers/#{customer_id}" }

  let(:connection_data) do
    {
      url: "https://example.com",
      consumer_key: "ck_123",
      consumer_secret: "cs_123"
    }.to_json
  end

  # Use strings as keys to match JSON parsing behavior
  let(:input_data) do
    {
      "customerId" => customer_id,
      "first_name" => "John",
      "last_name" => "Doe",
      "email" => "john@example.com",
      "empty_field" => ""
    }
  end

  let(:context) do
    double('ActionContext',
      connection: double('Connection', serialized_data: connection_data),
      serialized_input: input_data.to_json
    )
  end

  # This helper must match the logic in the execute method exactly
  let(:expected_cleaned_body) do
    {
      "first_name" => "John",
      "last_name" => "Doe",
      "email" => "john@example.com"
    }
  end

  before do
    stub_const("ApiClient", Class.new)
    allow(ApiClient).to receive(:new).and_return(api_client)
  end

  context "when request is successful" do
    let(:response_body) do
      {
        "id" => 123,
        "name" => "John Doe",
        "email" => "john@example.com"
      }.to_json
    end

    it "sends a PUT request with correct data and returns JSON" do
      # Now both endpoint and body match the implementation
      expect(api_client).to receive(:put).with(endpoint, expected_cleaned_body).and_return([200, response_body])

      result = UpdateCustomerAction.execute(context)

      expect(result["id"]).to eq(123)
      expect(result["name"]).to eq("John Doe")
      expect(result["email"]).to eq("john@example.com")
    end
  end

  context "when connection data is invalid" do
    let(:connection_data) { "invalid-json" }

    it "raises a connection configuration error" do
      expect {
        UpdateCustomerAction.execute(context)
      }.to raise_error(StandardError, /Invalid connection configuration/)
    end
  end

  context "when WooCommerce returns an error" do
    it "raises an error with status code" do
      allow(api_client).to receive(:put).and_return([400, "Bad Request"])

      expect {
        UpdateCustomerAction.execute(context)
      }.to raise_error(StandardError, /WooCommerce returned error code 400: Bad Request/)
    end
  end

  context "when customerId is missing in input" do
    let(:input_data) { { first_name: "Only Name" } }

    it "raises a validation error for the path parameter" do
      expect {
        UpdateCustomerAction.execute(context)
      }.to raise_error(StandardError, /customerId parameter is required/)
    end
  end

  context "when API response is malformed JSON" do
    it "raises a MalformedResponse error" do
      allow(api_client).to receive(:put).and_return([200, "not json"])

      expect {
        UpdateCustomerAction.execute(context)
      }.to raise_error(StandardError, /Failed to parse JSON response/)
    end
  end
end
