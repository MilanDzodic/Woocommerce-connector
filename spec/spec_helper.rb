# frozen_string_literal: true

require 'app_bridge'
require 'rspec-benchmark'
require 'json'
require 'net/http'
require 'uri'
require_relative 'test_helper'

RSpec.configure do |config|
  config.example_status_persistence_file_path = '.rspec_status'
  config.disable_monkey_patching!

  config.expect_with :rspec do |c|
    c.syntax = :expect
  end

  config.include RSpec::Benchmark::Matchers

  config.before(:suite) do
    @mock_server = TestHelper.create_mock_server
    @mock_server.start
  end

  config.after(:suite) do
    @mock_server&.stop
  end

  config.before(:each) do
    @mock_server ||= TestHelper.create_mock_server
  end
end
