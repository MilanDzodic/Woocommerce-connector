# frozen_string_literal: true

require 'spec_helper'

RSpec.describe 'actions.update_order' do
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

  it 'successfully updates an order' do
    input = {
      'orderId' => 123,
      'status' => 'completed'
    }

    mock_server.mock_endpoint(:put, '/orders/123', {
                                'id' => 123,
                                'status' => 'completed'
                              }, status: 200)

    result = tester.execute_action('update_order', input)
    expect(result).to be_a(AppBridge::ActionResponse)

    parsed_output = JSON.parse(result.serialized_output)
    expect(parsed_output['id']).to eq(123)
    expect(parsed_output['status']).to eq('completed')
  end

  it 'returns an error when the order is not found' do
    input = {
      'orderId' => 999,
      'status' => 'completed'
    }

    mock_server.mock_endpoint(:put, '/orders/999', {
                                'code' => 'woocommerce_rest_order_invalid_id',
                                'message' => 'Invalid ID.'
                              }, status: 404)

    expect do
      tester.execute_action('update_order', input)
    end.to raise_error(AppBridge::OtherError)
  end
end
