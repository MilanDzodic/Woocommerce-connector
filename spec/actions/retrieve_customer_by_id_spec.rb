require 'spec_helper'

RSpec.describe 'actions.retrieve_customer_by_id' do
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
        'headers' =>  { 'Authorization': 'Basic abc',
                        'Accept': 'application/json',
                        'Content-Type': 'application/json'
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

  it 'Retrieving customer with correct id and returning correct data from rust' do
    mock_server.mock_endpoint(:get, '/customers/123', {
      'id' => 123,
      'email' => 'customer@example.com',
      'first_name' => 'Anders',
      'last_name' => 'Andersson'
    })

    response = tester.execute_action('retrieve_customer_by_id', { 'customerId' => 123 })

    data = JSON.parse(response.serialized_output)

    puts "\n" + "="*40
    puts "DATA FRÅN RUST-ACTION (WASM):"
    pp data
    puts "="*40

    expect(data['id']).to eq(123)
    expect(data['email']).to eq('customer@example.com')
    expect(data['first_name']).to eq('Anders')
  end

  it 'Accepting both string and intiger as customer id in rust logic' do
    mock_server.mock_endpoint(:get, '/customers/456', { 'id' => 456 })
    res_str = tester.execute_action('retrieve_customer_by_id', { 'customerId' => '456' })
    expect(JSON.parse(res_str.serialized_output)['id']).to eq(456)

    mock_server.mock_endpoint(:get, '/customers/789', { 'id' => 789 })
    res_int = tester.execute_action('retrieve_customer_by_id', { 'customerId' => 789 })
    expect(JSON.parse(res_int.serialized_output)['id']).to eq(789)
  end

  it 'returns an empty hash when customer is not found and strategy is continue' do
    mock_server.mock_endpoint(:get, '/customers/999', { 'error' => 'Not Found' }, status: 404)

    result = tester.execute_action('retrieve_customer_by_id', {
      'customerId' => 999,
      'on_not_found' => 'continue'
    })

    expect(result.serialized_output).to eq('{}')
  end

  it 'raises CompleteParentException when customer is not found and strategy is exit_level' do
    mock_server.mock_endpoint(:get, '/customers/000', { 'error' => 'Not Found' }, status: 404)

    expect {
      tester.execute_action('retrieve_customer_by_id', {
        'customerId' => '000',
        'on_not_found' => 'exit_level'
      })
    }.to raise_error(AppBridge::CompleteParentException)
  end

  it 'raises CompleteWorkflowException when customer is not found and strategy is exit_execution' do
    mock_server.mock_endpoint(:get, '/customers/000', { 'error' => 'Not Found' }, status: 404)

    expect {
      tester.execute_action('retrieve_customer_by_id', {
        'customerId' => '000',
        'on_not_found' => 'exit_execution'
      })
    }.to raise_error(AppBridge::CompleteWorkflowException)
  end

  it 'Raises an error when customer is not found and strategy is fail' do
    mock_server.mock_endpoint(:get, '/customers/000', { 'error' => 'Not Found' }, status: 404)

    expect {
      tester.execute_action('retrieve_customer_by_id', {
        'customerId' => '000',
        'on_not_found' => 'fail'
      })
    }.to raise_error(AppBridge::OtherError, /Customer not found \(404\)/)
  end

  it 'returns a customer when a valid ID is provided' do
    input = { 'customerId' => 456 }

    mock_server.mock_endpoint(:get, '/customers/456', {
      'id' => 456,
      'email' => 'kalle@kula.se'
    }, status: 200)

    response = tester.execute_action('retrieve_customer_by_id', input)
    data = JSON.parse(response.serialized_output)

    expect(data['id']).to eq(456)
    expect(data['email']).to eq('kalle@kula.se')
  end

  it 'raises an error when customer is not found' do
    input = { 'customerId' => 999 }
    mock_server.mock_endpoint(:get, '/customers/999', { 'message' => 'Not found' }.to_json, status: 404)

    expect {
      tester.execute_action('retrieve_customer_by_id', input)
    }.to raise_error(AppBridge::OtherError, /404/)
  end
end
