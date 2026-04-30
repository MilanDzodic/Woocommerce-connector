#![recursion_limit = "512"]

mod actions;
mod client;
mod triggers;

include!("schemas/generated/action_routing.rs");

include!("schemas/generated/trigger_routing.rs");

include!("schemas/generated/embedded_schemas.rs");

use wit_bindgen::generate;

generate!({
    path: "wit/standout-app.wit",
    world: "bridge",
});

use crate::exports::standout::app::actions::{AppError as ActionsAppError, Guest as ActionsGuest};
use crate::exports::standout::app::triggers::{
    AppError as TriggersAppError, Guest as TriggersGuest,
};
use crate::standout::app::types::{ActionContext, ActionResponse, TriggerContext, TriggerResponse};

struct App;

impl TriggersGuest for App {
    fn trigger_ids() -> Result<Vec<String>, TriggersAppError> {
        Ok(get_available_triggers())
    }

    fn input_schema(context: TriggerContext) -> Result<String, TriggersAppError> {
        let schema_value = execute_trigger_input_schema_dynamically(&context.trigger_id, &context)
            .map_err(|e| TriggersAppError {
                code: e.code,
                message: e.message,
            })?;

        serde_json::to_string_pretty(&schema_value).map_err(|e| TriggersAppError {
            code: crate::standout::app::types::ErrorCode::Other,
            message: format!("Failed to serialize schema: {}", e),
        })
    }

    fn output_schema(context: TriggerContext) -> Result<String, TriggersAppError> {
        let schema_value = execute_trigger_output_schema_dynamically(&context.trigger_id, &context)
            .map_err(|e| TriggersAppError {
                code: e.code,
                message: e.message,
            })?;

        serde_json::to_string_pretty(&schema_value).map_err(|e| TriggersAppError {
            code: crate::standout::app::types::ErrorCode::Other,
            message: format!("Failed to serialize schema: {}", e),
        })
    }

    fn fetch_events(context: TriggerContext) -> Result<TriggerResponse, TriggersAppError> {
        let trigger_id = context.trigger_id.clone();
        execute_trigger_fetch_events_dynamically(&trigger_id, context).map_err(|e| {
            TriggersAppError {
                code: e.code,
                message: e.message,
            }
        })
    }
}

impl ActionsGuest for App {
    fn action_ids() -> Result<Vec<String>, ActionsAppError> {
        Ok(get_available_actions())
    }

    fn input_schema(context: ActionContext) -> Result<String, ActionsAppError> {
        let schema_value = execute_action_input_schema_dynamically(&context.action_id, &context)
            .map_err(|e| ActionsAppError {
                code: e.code,
                message: e.message,
            })?;

        serde_json::to_string_pretty(&schema_value).map_err(|e| ActionsAppError {
            code: crate::standout::app::types::ErrorCode::Other,
            message: format!("Failed to serialize schema: {}", e),
        })
    }

    fn output_schema(context: ActionContext) -> Result<String, ActionsAppError> {
        let schema_value = execute_action_output_schema_dynamically(&context.action_id, &context)
            .map_err(|e| ActionsAppError {
            code: e.code,
            message: e.message,
        })?;

        serde_json::to_string_pretty(&schema_value).map_err(|e| ActionsAppError {
            code: crate::standout::app::types::ErrorCode::Other,
            message: format!("Failed to serialize schema: {}", e),
        })
    }

    fn execute(context: ActionContext) -> Result<ActionResponse, ActionsAppError> {
        let action_id = context.action_id.clone();
        let result =
            execute_action_dynamically(&action_id, context).map_err(|e| ActionsAppError {
                code: e.code,
                message: e.message,
            })?;

        let serialized_output = serde_json::to_string(&result).map_err(|_e| ActionsAppError {
            code: crate::standout::app::types::ErrorCode::Other,
            message: "Failed to serialize response".to_string(),
        })?;

        Ok(ActionResponse { serialized_output })
    }
}

export!(App);
