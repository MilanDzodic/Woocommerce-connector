require 'spec_helper'

RSpec.describe 'actions.retrieve_customer_by_id' do
  let(:mock_server) { instance_variable_get(:@mock_server) }

  # Ladda WASM-appen (matchar din build)
  let(:app) do
    AppBridge::App.new('target/wasm32-wasip2/release/woocommerce_connector.wasm')
  end

  # Skapa Connection-objektet med nödvändiga headers för AppBridge
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
    # 1. Förbered mock-svar
    mock_server.mock_endpoint(:get, '/customers/123', {
      'id' => 123,
      'email' => 'customer@example.com',
      'first_name' => 'Anders',
      'last_name' => 'Andersson'
    })

    # 2. Kör actionen
    response = tester.execute_action('retrieve_customer_by_id', { 'customerId' => 123 })

    # 3. Parsa resultatet med den korrekta metoden :serialized_output
    data = JSON.parse(response.serialized_output)

    # Snygg utskrift för bekräftelse
    puts "\n" + "="*40
    puts "DATA FRÅN RUST-ACTION (WASM):"
    pp data
    puts "="*40

    # 4. Verifiera resultatet
    expect(data['id']).to eq(123)
    expect(data['email']).to eq('customer@example.com')
    expect(data['first_name']).to eq('Anders')
  end

  it 'Accepting both string and intiger as customer id in rust logic' do
    # Testa med sträng
    mock_server.mock_endpoint(:get, '/customers/456', { 'id' => 456 })
    res_str = tester.execute_action('retrieve_customer_by_id', { 'customerId' => '456' })
    expect(JSON.parse(res_str.serialized_output)['id']).to eq(456)

    # Testa med siffra
    mock_server.mock_endpoint(:get, '/customers/789', { 'id' => 789 })
    res_int = tester.execute_action('retrieve_customer_by_id', { 'customerId' => 789 })
    expect(JSON.parse(res_int.serialized_output)['id']).to eq(789)
  end

  it 'Handling with null at 404 not found from Woocommerce' do
    mock_server.mock_endpoint(:get, '/customers/999', { 
      'code' => 'rest_user_invalid_id', 
      'message' => 'Invalid ID.' 
    }, status: 404)
      
    result = tester.execute_action('retrieve_customer_by_id', { 
      'customerId' => 999, 
      'on_not_found' => 'continue' 
    })

    expect(result.serialized_output).to eq('null')
  end

  it 'Returns exit_level signal when customer is not found and strategy is exit_level' do
    mock_server.mock_endpoint(:get, '/customers/000', { 'error' => 'Not Found' }, status: 404)
    
    result = tester.execute_action('retrieve_customer_by_id', { 
      'customerId' => '000', 
      'on_not_found' => 'exit_level' 
    })

    data = JSON.parse(result.serialized_output)
    expect(data['status']).to eq('exit_level')
    expect(data['data']).to be_nil
  end

  it 'Returns exit_execution signal when customer is not found and strategy is exit_execution' do
    mock_server.mock_endpoint(:get, '/customers/000', { 'error' => 'Not Found' }, status: 404)
    
    result = tester.execute_action('retrieve_customer_by_id', { 
      'customerId' => '000', 
      'on_not_found' => 'exit_execution' 
    })

    data = JSON.parse(result.serialized_output)
    expect(data['status']).to eq('exit_execution')
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
end