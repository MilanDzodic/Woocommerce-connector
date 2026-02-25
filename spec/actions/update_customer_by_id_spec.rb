require 'spec_helper'

RSpec.describe 'actions.update_customer_by_id' do
  let(:mock_server) { instance_variable_get(:@mock_server) }

  let(:app) do
    AppBridge::App.new('target/wasm32-wasip2/release/woocommerce_connector.wasm')
  end

  let(:connection) do
    AppBridge::Connection.new(
      'test-id',
      'WooCommerce Connection',
      {
        'base_url' => 'http://localhost:8080',
        'headers' => {
          'Authorization' => 'Basic abc',
          'Accept' => 'application/json',
          'Content-Type' => 'application/json'
        }
      }.to_json
    )
  end

  let(:tester) do
    TestHelper::ActionTester.new(app, connection)
  end

  before do
    mock_server.clear_endpoints
  end

  let(:customer_id) { "123" }
  let(:input_data) do
    {
      "customerId" => customer_id,
      "first_name" => "John",
      "last_name" => "Doe",
      "email" => "john@example.com",
      "empty_field" => ""
    }
  end

  it 'updates customer with correct data and returns the response from rust' do
    # Expected body based on Rust logic (no customerId, no empty fields)
    expected_body = {
      "first_name" => "John",
      "last_name" => "Doe",
      "email" => "john@example.com"
    }

    mock_server.mock_endpoint(:put, "/customers/#{customer_id}", {
      'id' => 123,
      'first_name' => 'John',
      'last_name' => 'Doe',
      'email' => 'john@example.com'
    })

    response = tester.execute_action('update_customer_by_id', input_data)
    data = JSON.parse(response.serialized_output)

    expect(data['id']).to eq(123)
    expect(data['first_name']).to eq('John')
  end

  it 'raises an error when WooCommerce returns 400' do
    mock_server.mock_endpoint(:put, "/customers/#{customer_id}", {
      'message' => 'Bad Request'
    }, status: 400)

    expect {
      tester.execute_action('update_customer_by_id', input_data)
    }.to raise_error(AppBridge::OtherError, /400/)
  end

  it 'fails when customerId is missing in input' do
    invalid_input = { "first_name" => "John" }

    expect {
      tester.execute_action('update_customer_by_id', invalid_input)
    }.to raise_error(AppBridge::MisconfiguredError, /customerId parameter is required/)
  end
end
