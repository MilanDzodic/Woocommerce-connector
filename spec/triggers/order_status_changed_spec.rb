# frozen_string_literal: true

require 'spec_helper'

RSpec.describe 'triggers.order_status_changed' do
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

  it 'successfully fetches orders and generates combined unique IDs tracking status changes' do
    mock_server.mock_endpoint(:get, '/orders', [
                                { 'id' => 205, 'status' => 'completed' }
                              ], status: 200)

    result = tester.fetch_events('order_status_changed', {}, {})
    expect(result).to be_a(AppBridge::TriggerResponse)

    expect(result.events.length).to eq(1)

    # Event ID needs to include the status so the system recognizes status changes as new events
    expect(result.events.first.id).to eq('205-completed')

    event_data = JSON.parse(result.events.first.serialized_data)
    expect(event_data['item']['id']).to eq(205)
    expect(event_data['item']['status']).to eq('completed')
  end

  it 'returns an error if the WooCommerce API fails' do
    mock_server.mock_endpoint(:get, '/orders', { 'error' => 'Internal Server Error' }, status: 500)

    expect do
      tester.fetch_events('order_status_changed', {}, {})
    end.to raise_error(AppBridge::OtherError, /API error: 500/)
  end
end
