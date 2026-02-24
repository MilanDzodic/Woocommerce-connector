// Auto-generated file - do not edit manually
// Generated from existing action executors at compile time

// Action utilities
pub mod utils;

// Include generated action executors
pub mod create_customer {
    include!("../actions/create_customer/action.rs");
}

pub mod retrieve_customer_by_id {
    include!("../actions/retrieve_customer_by_id/action.rs");
}

pub mod search_products {
    include!("../actions/search_products/action.rs");
}

pub mod update_customer_by_id {
    include!("../actions/update_customer_by_id/action.rs");
}



