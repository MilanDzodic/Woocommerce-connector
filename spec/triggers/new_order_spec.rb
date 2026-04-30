# frozen_string_literal: true

require 'spec_helper'

RSpec.describe 'triggers.new_order' do
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
    TestHelper::TriggerTester.new(app, connection_data)
  end

  before do
    mock_server.clear_endpoints
  end

  it 'successfully fetches new orders and returns them as events' do
    mock_server.mock_endpoint(:get, '/orders', [
                                { 'id' => 101, 'status' => 'processing' },
                                { 'id' => 102, 'status' => 'pending' }
                              ], status: 200)

    result = tester.fetch_events('new_order', {}, {})
    expect(result).to be_a(AppBridge::TriggerResponse)

    expect(result.events.length).to eq(2)
    expect(result.events.first.id).to eq('101')

    event_data = JSON.parse(result.events.first.serialized_data)
    expect(event_data['item']['id']).to eq(101)
  end

  it 'returns an error if the WooCommerce API fails' do
    mock_server.mock_endpoint(:get, '/orders', { 'error' => 'Unauthorized' }, status: 401)

    expect do
      tester.fetch_events('new_order', {}, {})
    end.to raise_error(AppBridge::OtherError, /API error: 401/)
  end
end
