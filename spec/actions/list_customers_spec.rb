# frozen_string_literal: true

require 'spec_helper'

RSpec.describe 'actions.list_customers' do
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

  it 'successfully fetches customers with query parameters' do
    input = {
      'role' => 'administrator',
      'page' => 1,
      'per_page' => 10
    }

    mock_server.mock_endpoint(:get, '/customers?page=1&per_page=100&role=administrator', [
                                { 'id' => 1, 'role' => 'administrator' },
                                { 'id' => 2, 'role' => 'administrator' }
                              ], status: 200)

    result = tester.execute_action('list_customers', input)
    expect(result).to be_a(AppBridge::ActionResponse)

    parsed_output = JSON.parse(result.serialized_output)
    expect(parsed_output['items']).to be_an(Array)
    expect(parsed_output['items'].length).to eq(2)
    expect(parsed_output['items'].first['id']).to eq(1)
  end

  it 'successfully fetches all customers when no parameters are provided' do
    input = {}

    mock_server.mock_endpoint(:get, '/customers?page=1&per_page=100', [
                                { 'id' => 1 },
                                { 'id' => 2 },
                                { 'id' => 3 }
                              ], status: 200)

    result = tester.execute_action('list_customers', input)
    parsed_output = JSON.parse(result.serialized_output)

    expect(parsed_output['items'].length).to eq(3)
  end

  it 'returns an error on unauthorized access' do
    input = {}

    mock_server.mock_endpoint(:get, '/customers?page=1&per_page=100', {
                                'code' => 'woocommerce_rest_cannot_view',
                                'message' => 'Sorry, you cannot list resources.'
                              }, status: 401)

    expect do
      tester.execute_action('list_customers', input)
    end.to raise_error(AppBridge::OtherError)
  end

  it 'paginates through multiple pages until all customers are fetched' do
    page1_customers = Array.new(100) { |i| { 'id' => i, 'name' => "Customer #{i}" } }
    page2_customers = Array.new(5) { |i| { 'id' => i + 100, 'name' => "Customer #{i + 100}" } }

    mock_server.mock_endpoint(:get, '/customers?page=1&per_page=100', page1_customers)
    mock_server.mock_endpoint(:get, '/customers?page=2&per_page=100', page2_customers)

    response = tester.execute_action('list_customers', {})
    data = JSON.parse(response.serialized_output)

    expect(data['items'].length).to eq(105)
    expect(data['items'].first['id']).to eq(0)
    expect(data['items'].last['id']).to eq(104)
  end
end
