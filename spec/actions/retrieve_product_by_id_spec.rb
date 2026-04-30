# frozen_string_literal: true

require 'spec_helper'

RSpec.describe 'actions.retrieve_product_by_id' do
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
        'headers' => { Authorization: 'Basic abc',
                       Accept: 'application/json',
                       'Content-Type': 'application/json' }
      }.to_json
    )
  end

  let(:tester) do
    TestHelper::ActionTester.new(app, connection)
  end

  before do
    mock_server.clear_endpoints
  end

  it 'Retrieving product with correct id and returning correct data from rust' do
    mock_server.mock_endpoint(:get, '/products/123', {
                                'id' => 123,
                                'name' => 'Premium Shirt',
                                'status' => 'publish',
                                'price' => '29.99'
                              })

    response = tester.execute_action('retrieve_product_by_id', { 'productId' => 123 })

    data = JSON.parse(response.serialized_output)

    expect(data['id']).to eq(123)
    expect(data['name']).to eq('Premium Shirt')
    expect(data['status']).to eq('publish')
  end

  it 'Accepting both string and integer as product id in rust logic' do
    mock_server.mock_endpoint(:get, '/products/456', { 'id' => 456 })
    res_str = tester.execute_action('retrieve_product_by_id', { 'productId' => '456' })
    expect(JSON.parse(res_str.serialized_output)['id']).to eq(456)

    mock_server.mock_endpoint(:get, '/products/789', { 'id' => 789 })
    res_int = tester.execute_action('retrieve_product_by_id', { 'productId' => 789 })
    expect(JSON.parse(res_int.serialized_output)['id']).to eq(789)
  end

  it 'returns an empty hash when product is not found and strategy is continue' do
    mock_server.mock_endpoint(:get, '/products/999', { 'error' => 'Not Found' }, status: 404)

    result = tester.execute_action('retrieve_product_by_id', {
                                     'productId' => 999,
                                     'on_not_found' => 'continue'
                                   })

    expect(result.serialized_output).to eq('{}')
  end

  it 'raises CompleteParentException when product is not found and strategy is exit_level' do
    mock_server.mock_endpoint(:get, '/products/000', { 'error' => 'Not Found' }, status: 404)

    expect do
      tester.execute_action('retrieve_product_by_id', {
                              'productId' => '000',
                              'on_not_found' => 'exit_level'
                            })
    end.to raise_error(AppBridge::CompleteParentException)
  end

  it 'raises CompleteWorkflowException when product is not found and strategy is exit_execution' do
    mock_server.mock_endpoint(:get, '/products/000', { 'error' => 'Not Found' }, status: 404)

    expect do
      tester.execute_action('retrieve_product_by_id', {
                              'productId' => '000',
                              'on_not_found' => 'exit_execution'
                            })
    end.to raise_error(AppBridge::CompleteWorkflowException)
  end

  it 'Raises an error when product is not found and strategy is fail' do
    mock_server.mock_endpoint(:get, '/products/000', { 'error' => 'Not Found' }, status: 404)

    expect do
      tester.execute_action('retrieve_product_by_id', {
                              'productId' => '000',
                              'on_not_found' => 'fail'
                            })
    end.to raise_error(AppBridge::OtherError, /Product not found \(404\)/)
  end

  it 'returns a product when a valid ID is provided' do
    input = { 'productId' => 456 }

    mock_server.mock_endpoint(:get, '/products/456', {
                                'id' => 456,
                                'name' => 'Awesome Coffee Mug'
                              }, status: 200)

    response = tester.execute_action('retrieve_product_by_id', input)
    data = JSON.parse(response.serialized_output)

    expect(data['id']).to eq(456)
    expect(data['name']).to eq('Awesome Coffee Mug')
  end
end
