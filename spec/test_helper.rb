# frozen_string_literal: true

require 'app_bridge'
require 'json'
require 'net/http'
require 'uri'

module TestHelper
  class MockServer
    def initialize(base_url = 'http://localhost:8080')
      @base_url = base_url
      @configured_endpoints = []
    end

    def start
      return false unless system('docker --version > /dev/null 2>&1')

      system('./scripts/test-setup.sh start > /dev/null 2>&1')

      max_attempts = 30
      attempts = 0

      while attempts < max_attempts
        begin
          response = Net::HTTP.get_response(URI("#{@base_url}/__admin/health"))
          return true if response.code == '200'
        rescue StandardError
          # Server not ready yet
        end
        sleep 1
        attempts += 1
      end
      false
    end

    def stop
      system('docker compose -f docker-compose.test.yml down > /dev/null 2>&1')
    end

    def mock_endpoint(method, path, response_body, status: 200, headers: {})
      mock_config = {
        'request' => {
          'method' => method.to_s.upcase,
          'url' => path
        },
        'response' => {
          'status' => status,
          'headers' => headers.merge('Content-Type' => 'application/json'),
          'body' => JSON.generate(response_body)
        }
      }
      configure_endpoint(mock_config)
    end

    def mock_endpoint_pattern(method, url_pattern, response_body, status: 200, headers: {})
      mock_config = {
        'request' => {
          'method' => method.to_s.upcase,
          'urlPattern' => url_pattern
        },
        'response' => {
          'status' => status,
          'headers' => headers.merge('Content-Type' => 'application/json'),
          'body' => JSON.generate(response_body)
        }
      }
      configure_endpoint(mock_config)
    end

    def clear_endpoints
      uri = URI("#{@base_url}/__admin/mappings")
      http = Net::HTTP.new(uri.host, uri.port)
      request = Net::HTTP::Delete.new(uri)
      http.request(request)
      @configured_endpoints.clear
    rescue StandardError
      # Ignore errors during cleanup
    end

    private

    def configure_endpoint(mock_config)
      uri = URI("#{@base_url}/__admin/mappings")
      http = Net::HTTP.new(uri.host, uri.port)
      request = Net::HTTP::Post.new(uri)
      request['Content-Type'] = 'application/json'
      request.body = JSON.generate(mock_config)

      response = http.request(request)
      if response.code == '201'
        @configured_endpoints << mock_config
        true
      else
        false
      end
    end
  end

  class ActionTester
    def initialize(app, connection_data)
      @app = app
      # Smart hantering: Om testet redan skickar in ett AppBridge-objekt använder vi det,
      # annars bygger vi ett automatiskt (som i Jira).
      @connection = if connection_data.is_a?(AppBridge::Connection)
                      connection_data
                    else
                      AppBridge::Connection.new(
                        'test-connection',
                        'Test Connection',
                        connection_data.to_json
                      )
                    end
    end

    def execute_action(action_name, input_data)
      action_context = AppBridge::ActionContext.new(
        action_name,
        @connection,
        input_data.to_json
      )
      @app.execute_action(action_context)
    end

    def get_input_schema(action_name)
      action_context = AppBridge::ActionContext.new(
        action_name,
        @connection,
        '{}'
      )
      @app.action_input_schema(action_context)
    end

    def get_output_schema(action_name)
      action_context = AppBridge::ActionContext.new(
        action_name,
        @connection,
        '{}'
      )
      @app.action_output_schema(action_context)
    end
  end

  class TriggerTester
    def initialize(app, connection_data)
      @app = app
      @connection = if connection_data.is_a?(AppBridge::Connection)
                      connection_data
                    else
                      AppBridge::Connection.new(
                        'test-connection',
                        'Test Connection',
                        connection_data.to_json
                      )
                    end
    end

    def fetch_events(trigger_name, input_data = {}, store_data = {})
      trigger_context = AppBridge::TriggerContext.new(
        trigger_name,
        @connection,
        store_data.to_json,
        input_data.to_json
      )
      @app.fetch_events(trigger_context)
    end

    def get_input_schema(trigger_name)
      trigger_context = AppBridge::TriggerContext.new(trigger_name, @connection, '{}', '{}')
      @app.trigger_input_schema(trigger_context)
    end

    def get_output_schema(trigger_name)
      trigger_context = AppBridge::TriggerContext.new(trigger_name, @connection, '{}', '{}')
      @app.trigger_output_schema(trigger_context)
    end
  end

  def self.create_mock_server
    MockServer.new
  end

  def self.create_action_tester(app, connection_data)
    ActionTester.new(app, connection_data)
  end

  def self.create_trigger_tester(app, connection_data)
    TriggerTester.new(app, connection_data)
  end

  def self.base_connection
    {
      'base_url' => 'http://localhost:8080',
      'headers' => {
        'Authorization' => 'Basic abc',
        'Content-Type' => 'application/json'
      }
    }
  end
end
