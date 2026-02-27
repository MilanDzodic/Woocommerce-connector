require 'spec_helper'

RSpec.describe 'actions.list_all_products' do
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

    mock_server.mock_endpoint(:get, url, [
      { 'id' => 555, 'sku' => 'GTX-1080-TI' }
    ])

    response = tester.execute_action('search_products', { 'sku' => sku })
    data = JSON.parse(response.serialized_output)

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

  it 'paginates through multiple pages until all products are fetched' do

    page1_products = Array.new(100) { |i| { 'id' => i, 'name' => "Product #{i}" } }

    page2_products = Array.new(5) { |i| { 'id' => i + 100, 'name' => "Product #{i + 100}" } }

    mock_server.mock_endpoint(:get, "/products?page=1&per_page=100&status=publish", page1_products)

    mock_server.mock_endpoint(:get, "/products?page=2&per_page=100&status=publish", page2_products)

    response = tester.execute_action('search_products', { 'status' => 'publish' })
    data = JSON.parse(response.serialized_output)

    expect(data['items'].length).to eq(105)
    expect(data['items'].first['id']).to eq(0)
    expect(data['items'].last['id']).to eq(104)
  end

  it 'returns an empty items list when no products are found' do
    url = "/products?page=1&per_page=100&sku=MISSING"
    mock_server.mock_endpoint(:get, url, [])

    response = tester.execute_action('search_products', { 'sku' => 'MISSING' })
    data = JSON.parse(response.serialized_output)

    expect(data['items']).to eq([])
  end
end
