require 'spec_helper'

RSpec.describe 'actions.create_customer' do
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

  it 'successfully creates a customer and returns the correct data from rust' do
    input = {
      'email' => 'ny_kund@example.com',
      'first_name' => 'Test',
      'last_name' => 'Person'
    }

    mock_server.mock_endpoint(:post, '/customers', {
      'id' => 456,
      'email' => 'ny_kund@example.com',
      'first_name' => 'Test',
      'last_name' => 'Person'
    }, status: 201)

    response = tester.execute_action('create_customer', input)
    data = JSON.parse(response.serialized_output)

    expect(data['id']).to eq(456)
    expect(data['email']).to eq('ny_kund@example.com')
    expect(data['first_name']).to eq('Test')
  end

  it 'raises an error when WooCommerce returns a 400 Bad Request' do
    input = { 'email' => 'invalid-email' }

    mock_server.mock_endpoint(:post, '/customers', {
      'code' => 'registration-error-invalid-email',
      'message' => 'Please provide a valid email address.'
    }, status: 400)

    expect {
      tester.execute_action('create_customer', input)
    }.to raise_error(AppBridge::OtherError, /WooCommerce returnerade felkod 400/)
  end

  it 'sends POST when ID is missing' do
    input = { 'email' => 'ny@test.se' }
    mock_server.mock_endpoint(:post, '/customers', { 'id' => 1 }.to_json, status: 201)
    tester.execute_action('create_customer', input)
  end

  it 'sends PUT when ID is provided' do
    input = { 'id' => 123, 'email' => 'befintlig@test.se' }
    mock_server.mock_endpoint(:put, '/customers/123', { 'id' => 123 }.to_json, status: 200)
    tester.execute_action('create_customer', input)
  end
end
