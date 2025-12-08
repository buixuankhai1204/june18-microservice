use chrono::NaiveTime;
use erp_backend::application::channel::channel_command::{
    CreateChannelCommand, UpdateChannelCommand,
};
use erp_backend::application::department::department_command::CreateDepartmentCommand;
use erp_backend::application::position::position_command::CreatePositionCommand;

/// Fixture for creating a test channel
pub fn create_test_channel_command() -> CreateChannelCommand {
    CreateChannelCommand {
        name: "VTV1".to_string(),
        description: Some("Test channel for unit tests".to_string()),
        logo_url: None,
        time_on: NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
        time_off: NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
    }
}

/// Fixture for creating another test channel
pub fn create_test_channel_command_2() -> CreateChannelCommand {
    CreateChannelCommand {
        name: "VTV2".to_string(),
        description: Some("Second test channel".to_string()),
        logo_url: None,
        time_on: NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
        time_off: NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
    }
}

/// Fixture for updating a channel
pub fn update_test_channel_command() -> UpdateChannelCommand {
    UpdateChannelCommand {
        name: Some("VTV1 Updated".to_string()),
        description: Some("Updated test channel".to_string()),
        logo_url: None,
        time_on: None,
        time_off: None,
    }
}

/// Fixture for creating a test department
pub fn create_test_department_command() -> CreateDepartmentCommand {
    CreateDepartmentCommand {
        name: "Test Department".to_string(),
        short_name: "TEST_DEPT".to_string(),
        is_socialize: false,
        description: Some("Test department for unit tests".to_string()),
        image_url: None,
    }
}

/// Fixture for creating a test position
pub fn create_test_position_command() -> CreatePositionCommand {
    CreatePositionCommand {
        name: "Test Position".to_string(),
        short_name: "TEST_POS".to_string(),
        description: Some("Test position for unit tests".to_string()),
    }
}
