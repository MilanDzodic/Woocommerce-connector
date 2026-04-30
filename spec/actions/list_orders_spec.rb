# frozen_string_literal: true

require 'spec_helper'

RSpec.describe 'actions.list_orders' do
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

  it 'successfully fetches orders with query parameters' do
    input = {
      'status' => 'processing',
      'page' => 1,
      'per_page' => 10
    }

    mock_server.mock_endpoint(:get, '/orders?page=1&per_page=10&status=processing', [
                                { 'id' => 1, 'status' => 'processing' },
                                { 'id' => 2, 'status' => 'processing' }
                              ], status: 200)

    result = tester.execute_action('list_orders', input)
    expect(result).to be_a(AppBridge::ActionResponse)

    parsed_output = JSON.parse(result.serialized_output)
    expect(parsed_output).to be_an(Array)
    expect(parsed_output.length).to eq(2)
    expect(parsed_output.first['id']).to eq(1)
  end

  it 'successfully fetches all orders when no parameters are provided' do
    input = {}

    mock_server.mock_endpoint(:get, '/orders', [
                                { 'id' => 1 },
                                { 'id' => 2 },
                                { 'id' => 3 }
                              ], status: 200)

    result = tester.execute_action('list_orders', input)
    parsed_output = JSON.parse(result.serialized_output)

    expect(parsed_output.length).to eq(3)
  end

  it 'returns an error on unauthorized access' do
    input = {}

    mock_server.mock_endpoint(:get, '/orders', {
                                'code' => 'woocommerce_rest_cannot_view',
                                'message' => 'Sorry, you cannot list resources.'
                              }, status: 401)

    expect do
      tester.execute_action('list_orders', input)
    end.to raise_error(AppBridge::OtherError)
  end
end
