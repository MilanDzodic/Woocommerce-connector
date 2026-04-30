# frozen_string_literal: true

require 'spec_helper'

RSpec.describe 'actions.create_or_update_product' do
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

  it 'successfully creates a product (POST) when productId is missing' do
    input = {
      'name' => 'Premium T-shirt',
      'type' => 'simple',
      'regular_price' => '299'
    }

    mock_server.mock_endpoint(:post, '/products', {
                                'id' => 101,
                                'name' => 'Premium T-shirt',
                                'type' => 'simple',
                                'price' => '299'
                              }, status: 201)

    response = tester.execute_action('create_or_update_product', input)
    data = JSON.parse(response.serialized_output)

    expect(data['id'].to_i).to eq(101)
    expect(data['name']).to eq('Premium T-shirt')
  end

  it 'successfully updates a product (PUT) when productId is provided' do
    input = {
      'productId' => 101,
      'name' => 'Updated T-shirt'
    }

    mock_server.mock_endpoint(:put, '/products/101', {
                                'id' => 101,
                                'name' => 'Updated T-shirt'
                              }, status: 200)

    response = tester.execute_action('create_or_update_product', input)
    data = JSON.parse(response.serialized_output)

    expect(data['id'].to_i).to eq(101)
    expect(data['name']).to eq('Updated T-shirt')
  end

  it 'raises an error when WooCommerce returns a 404 Not Found' do
    input = { 'productId' => 999, 'name' => 'Ghost Product' }

    mock_server.mock_endpoint(:put, '/products/999', {
                                'code' => 'woocommerce_rest_product_invalid_id',
                                'message' => 'Invalid ID.'
                              }, status: 404)

    expect do
      tester.execute_action('create_or_update_product', input)
    end.to raise_error(AppBridge::OtherError, /404/)
  end

  it 'raises an error when WooCommerce returns a 400 Bad Request' do
    input = { 'name' => '' }

    mock_server.mock_endpoint(:post, '/products', {
                                'code' => 'rest_invalid_param',
                                'message' => 'Invalid parameter(s): name'
                              }, status: 400)

    expect do
      tester.execute_action('create_or_update_product', input)
    end.to raise_error(AppBridge::OtherError, /400/)
  end
end
