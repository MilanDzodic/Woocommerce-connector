require 'spec_helper'

RSpec.describe 'actions.retrieve_product_by_sku' do
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

    # WooCommerce returns a list, but Rust code extracts the specific object
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

    # We expect a Hash (object), not an Array, due to the Rust code's filtering logic
    expect(data).to be_a(Hash)
    expect(data['sku']).to eq(sku)
    expect(data['id']).to eq(555)
  end

  it 'handles multiple query parameters correctly' do
    params = {
      'sku' => 'BLUE-SHIRT',
      'status' => 'publish',
      'per_page' => 1
    }

    # IMPORTANT: Rust sends parameters in the order they are defined in the params vector:
    # per_page (index 2), status (index 17), sku (index 23)
    # URL: /products?per_page=1&status=publish&sku=BLUE-SHIRT
    mock_server.mock_endpoint(:get, "/products?per_page=1&status=publish&sku=BLUE-SHIRT", [
      { 'id' => 1, 'sku' => 'BLUE-SHIRT' }
    ])

    response = tester.execute_action('list_all_products', params)
    data = JSON.parse(response.serialized_output)

    # Since we provided an SKU, Rust returns a single object
    expect(data).to be_a(Hash)
    expect(data['sku']).to eq('BLUE-SHIRT')
  end

  it 'returns an empty object when no product matches the SKU and strategy is continue' do
    mock_server.mock_endpoint(:get, "/products?sku=NON-EXISTENT", [])

    # We send 'continue' to receive an empty hash instead of an error
    response = tester.execute_action('list_all_products', {
      'sku' => 'NON-EXISTENT',
      'on_not_found' => 'continue'
    })
    data = JSON.parse(response.serialized_output)

    expect(data).to eq({})
  end


    it 'raises CompleteParentException on 404 if strategy is exit_level' do
    # Simulate a 404 response from WooCommerce
    mock_server.mock_endpoint(:get, "/products?sku=FAIL", { 'error' => 'Not Found' }, status: 404)

    expect {
      tester.execute_action('list_all_products', {
        'sku' => 'FAIL',
        'on_not_found' => 'exit_level'
      })
    }.to raise_error(AppBridge::CompleteParentException)
    end
end
