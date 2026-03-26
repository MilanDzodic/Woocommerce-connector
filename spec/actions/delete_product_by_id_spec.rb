require 'spec_helper'

RSpec.describe 'actions.delete_product_by_id' do
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
        'headers' =>  {
          'Authorization': 'Basic abc',
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

  it 'deletes a product with the correct ID and returns data from Rust' do
    mock_server.mock_endpoint(:delete, '/products/123', {
      'id' => 123,
      'name' => 'Deleted Product',
      'status' => 'trash'
    })

    response = tester.execute_action('delete_product_by_id', { 'productId' => 123 })
    data = JSON.parse(response.serialized_output)

    expect(data['id']).to eq(123)
    expect(data['status']).to eq('trash')
  end

  it 'accepts both string and integer as productId in the Rust logic' do
    mock_server.mock_endpoint(:delete, '/products/456', { 'id' => 456 })
    res_str = tester.execute_action('delete_product_by_id', { 'productId' => '456' })
    expect(JSON.parse(res_str.serialized_output)['id']).to eq(456)

    mock_server.mock_endpoint(:delete, '/products/789', { 'id' => 789 })
    res_int = tester.execute_action('delete_product_by_id', { 'productId' => 789 })
    expect(JSON.parse(res_int.serialized_output)['id']).to eq(789)
  end

  it 'raises an error when the product to be deleted is not found (404)' do
    mock_server.mock_endpoint(:delete, '/products/999', {
      'code' => 'woocommerce_rest_not_found',
      'message' => 'Invalid ID'
    }, status: 404)

    expect {
      tester.execute_action('delete_product_by_id', { 'productId' => 999 })
    }.to raise_error(AppBridge::OtherError, /404/)
  end

  it 'raises an error if productId is completely missing from the input' do
    expect {
      tester.execute_action('delete_product_by_id', { 'not_an_id' => 'abc' })
    }.to raise_error(AppBridge::MisconfiguredError, /productId parameter is required/)
  end
end
