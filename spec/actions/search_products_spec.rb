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

    url = "/products?page=1&per_page=100&sku=#{sku}"

    # WooCommerce returns a list, Rust code filters it and wraps in { "items": [...] }
    mock_server.mock_endpoint(:get, url, [
      { 'id' => 555, 'sku' => 'GTX-1080-TI' }
    ])

    response = tester.execute_action('search_products', { 'sku' => sku })
    data = JSON.parse(response.serialized_output)

    # We now expect an object with an 'items' array (Feedback point 1)
    expect(data['items']).to be_an(Array)
    expect(data['items'].first['sku']).to eq(sku)
    expect(data['items'].first['id']).to eq(555)
  end

  it 'handles multiple query parameters correctly regardless of order' do
    params = {
      'sku' => 'BLUE-SHIRT',
      'status' => 'publish'
    }

    url = "/products?page=1&per_page=100&status=publish&sku=BLUE-SHIRT"

    mock_server.mock_endpoint(:get, url, [
      { 'id' => 1, 'sku' => 'BLUE-SHIRT' }
    ])

    response = tester.execute_action('search_products', params)
    data = JSON.parse(response.serialized_output)

    expect(data['items']).to be_an(Array)
    expect(data['items'].first['sku']).to eq('BLUE-SHIRT')
  end

  it 'returns an empty items list when no product matches the SKU and strategy is continue' do
    url = "/products?page=1&per_page=100&sku=NON-EXISTENT"
    mock_server.mock_endpoint(:get, url, [])

    response = tester.execute_action('search_products', { 'sku' => 'NON-EXISTENT', 'on_not_found' => 'continue' })

    data = JSON.parse(response.serialized_output)

    expect(data['items']).to eq([])
  end

  describe 'with flow control' do
    it 'raises CompleteParentException on 404 if strategy is exit_level' do
      url = "/products?page=1&per_page=100&sku=FAIL"
      body = { message: "Not Found" }
      mock_server.mock_endpoint(:get, url, body, status: 404)

      expect {
        tester.execute_action('search_products', {
          'sku' => 'FAIL',
          'on_not_found' => 'exit_level'
        })
      }.to raise_error(AppBridge::CompleteParentException)
    end

    it 'raises CompleteParentException if product is not found in result list' do
      url = "/products?page=1&per_page=100&sku=EMPTY-LIST"
      mock_server.mock_endpoint(:get, url, [])

      expect {
        tester.execute_action('search_products', {
          'sku' => 'EMPTY-LIST',
          'on_not_found' => 'exit_level'
        })
      }.to raise_error(AppBridge::CompleteParentException)
    end
  end
end
