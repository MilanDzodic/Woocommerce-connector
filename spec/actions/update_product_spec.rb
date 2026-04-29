# frozen_string_literal: true

require 'spec_helper'

RSpec.describe 'actions.update_product' do
  let(:mock_server) { instance_variable_get(:@mock_server) }

  let(:app) do
    AppBridge::App.new('target/wasm32-wasip2/release/woocommerce_connector.wasm')
  end

  let(:connection_data) do
    {
      'base_url' => 'http://localhost:8080',
      'headers' => {
        'Authorization' => 'Basic abc',
        'Content-Type' => 'application/json'
      }
    }
  end

  let(:tester) do
    TestHelper::ActionTester.new(app, connection_data)
  end

  before do
    mock_server.clear_endpoints
  end

  it 'successfully updates a product' do
    input = {
      'productId' => 101,
      'regular_price' => '29.99',
      'stock_quantity' => 50
    }

    mock_server.mock_endpoint(:put, '/products/101', {
                                'id' => 101,
                                'regular_price' => '29.99',
                                'stock_quantity' => 50
                              }, status: 200)

    result = tester.execute_action('update_product', input)
    expect(result).to be_a(AppBridge::ActionResponse)

    parsed_output = JSON.parse(result.serialized_output)
    expect(parsed_output['id']).to eq(101)
    expect(parsed_output['stock_quantity']).to eq(50)
  end

  it 'returns an error when invalid parameters are provided' do
    input = {
      'productId' => 101,
      'regular_price' => 'invalid_price'
    }

    mock_server.mock_endpoint(:put, '/products/101', {
                                'code' => 'rest_invalid_param',
                                'message' => 'Invalid parameter(s): regular_price'
                              }, status: 400)

    expect do
      tester.execute_action('update_product', input)
    end.to raise_error(AppBridge::OtherError)
  end
end
