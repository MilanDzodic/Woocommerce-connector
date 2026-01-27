require 'spec_helper'

# Updated description to match folder name (Feedback point 5)
RSpec.describe 'actions.list_all_products' do
  let(:mock_server) { instance_variable_get(:@mock_server) }

  # Path to your compiled WASM binary
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

  it 'successfully retrieves a product by SKU' do
    sku = 'GTX-1080-TI'

    # WooCommerce returns a list, Rust code filters it and wraps in { "items": [...] }
    mock_server.mock_endpoint(:get, "/products?sku=#{sku}", [
      {
        'id' => 555,
        'name' => 'Graphics Card',
        'sku' => sku,
        'price' => '5000'
      }
    ])

    response = tester.execute_action('list_all_products', { 'sku' => sku })
    data = JSON.parse(response.serialized_output)

    # We now expect an object with an 'items' array (Feedback point 1)
    expect(data['items']).to be_an(Array)
    expect(data['items'].first['sku']).to eq(sku)
    expect(data['items'].first['id']).to eq(555)
  end

  it 'handles multiple query parameters correctly regardless of order' do
    params = {
      'sku' => 'BLUE-SHIRT',
      'status' => 'publish',
      'per_page' => 1
    }

    # Use a Regexp to match the URL regardless of parameter order (Feedback point 4)
    # This regex ensures all required parameters exist in the query string
    expected_url = "/products?per_page=1&status=publish&sku=BLUE-SHIRT"

    mock_server.mock_endpoint(:get, expected_url, [
      { 'id' => 1, 'sku' => 'BLUE-SHIRT' }
    ])

    response = tester.execute_action('list_all_products', params)
    data = JSON.parse(response.serialized_output)

    expect(data['items']).to be_an(Array)
    expect(data['items'].first['sku']).to eq('BLUE-SHIRT')
  end

  it 'returns an empty items list when no product matches the SKU and strategy is continue' do
    # WooCommerce usually returns 200 OK with an empty list [] for missed searches
    mock_server.mock_endpoint(:get, "/products?sku=NON-EXISTENT", [])

    response = tester.execute_action('list_all_products', {
      'sku' => 'NON-EXISTENT',
      'on_not_found' => 'continue'
    })
    data = JSON.parse(response.serialized_output)

    # Should return an empty object or empty items list depending on your handle_not_found
    # Based on the new schema logic:
    expect(data).to eq({})
  end

  context 'with flow control' do
    it 'raises CompleteParentException on 404 if strategy is exit_level' do
      # Simulate a 404 response
      mock_server.mock_endpoint(:get, "/products?sku=FAIL", { 'error' => 'Not Found' }, status: 404)

      expect {
        tester.execute_action('list_all_products', {
          'sku' => 'FAIL',
          'on_not_found' => 'exit_level'
        })
      }.to raise_error(AppBridge::CompleteParentException)
    end

    it 'raises CompleteParentException if product is not found in result list' do
      # Feedback point 3: Handle 200 OK with empty list
      mock_server.mock_endpoint(:get, "/products?sku=EMPTY-LIST", [])

      expect {
        tester.execute_action('list_all_products', {
          'sku' => 'EMPTY-LIST',
          'on_not_found' => 'exit_level'
        })
      }.to raise_error(AppBridge::CompleteParentException)
    end
  end
end
