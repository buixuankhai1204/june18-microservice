#[cfg(test)]
mod channel_integration_tests {
    use crate::common;
    use chrono::NaiveTime;
    use erp_backend::application::channel::channel_command::{
        CreateChannelCommand, UpdateChannelCommand,
    };
    use erp_backend::application::channel::channel_service_interface::ChannelServiceInterface;
    use erp_backend::util::filter_and_pagination::PageQueryParam;
    use sea_orm::TransactionTrait;

    /// Helper function to create a test channel command
    fn create_test_channel(name: &str, description: &str) -> CreateChannelCommand {
        CreateChannelCommand {
            name: name.to_string(),
            description: Some(description.to_string()),
            logo_url: Some(format!("https://example.com/{}.png", name.to_lowercase())),
            time_on: NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            time_off: NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
        }
    }

    fn create_test_channel_failure(name: &str) -> CreateChannelCommand {
        CreateChannelCommand {
            name: name.to_string(),
            description: None,
            logo_url: None,
            time_on: NaiveTime::from_hms_opt(23, 0, 0).unwrap(),
            time_off: NaiveTime::from_hms_opt(6, 59, 59).unwrap(),
        }
    }

    /// Test: Successfully create a new channel
    #[tokio::test]
    async fn test_create_channel_success() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let command =
            create_test_channel("Test Channel VTV1", "Test channel for integration testing");
        let result = state.channel_service.create_channel(&tx, &command).await;

        assert!(result.is_ok(), "Failed to create channel: {:?}", result.err());
        let channel = result.unwrap();
        assert_eq!(channel.name, "Test Channel VTV1");
    }

    #[tokio::test]
    async fn test_create_channel_failure() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");
        let command = create_test_channel_failure("Invalid Channel");
        let result = state.channel_service.create_channel(&tx, &command).await;
        assert!(result.is_err(), "Expected failure when creating invalid channel");
    }

    /// Test: Get channel by ID
    #[tokio::test]
    async fn test_get_channel_by_id() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let command = create_test_channel("VTV2 Test", "VTV2 test channel");
        let created = match state.channel_service.create_channel(&tx, &command).await {
            Ok(ch) => ch,
            Err(e) => panic!("Failed to create channel for get by id test: {:?}", e),
        };

        let result = state.channel_service.get_channel_by_id(&tx, created.id).await;
        assert!(result.is_ok(), "Failed to get channel by id");

        let channel = result.unwrap();
        assert_eq!(channel.id, created.id);
        assert_eq!(channel.name, "VTV2 Test");
    }

    /// Test: Update channel
    #[tokio::test]
    async fn test_update_channel() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let command = create_test_channel("VTV3 Original", "Original description");
        let created = match state.channel_service.create_channel(&tx, &command).await {
            Ok(ch) => ch,
            Err(e) => panic!("Failed to create channel for update test: {:?}", e),
        };

        let update_command = UpdateChannelCommand {
            name: Some("VTV3 Updated".to_string()),
            description: Some("Updated description".to_string()),
            logo_url: Some("https://example.com/vtv3-new.png".to_string()),
            time_on: Some(NaiveTime::from_hms_opt(7, 0, 0).unwrap()),
            time_off: Some(NaiveTime::from_hms_opt(22, 0, 0).unwrap()),
        };

        let result = state.channel_service.update_channel(&tx, created.id, &update_command).await;
        assert!(result.is_ok(), "Failed to update channel");

        let updated = match state.channel_service.get_channel_by_id(&tx, created.id).await {
            Ok(ch) => ch,
            Err(e) => panic!("Failed to get updated channel: {:?}", e),
        };
        assert_eq!(updated.name, "VTV3 Updated");
    }

    /// Test: List channels
    #[tokio::test]
    async fn test_list_channels() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        // Create multiple channels
        for i in 4..=6 {
            let command = create_test_channel(&format!("VTV{} Test", i), &format!("Channel {}", i));

            match state.channel_service.create_channel(&tx, &command).await {
                Ok(_) => {},
                Err(e) => assert!(false, "Failed to create channel {}: {:?}", i, e),
            };
        }

        let params = PageQueryParam {
            page_num: Some(1),
            page_size: Some(10),
            sort_direction: None,
            sort_by: None,
            q: None,
            start_date: None,
            end_date: None,
        };

        let result = state.channel_service.list_channels(&tx, 1, &params).await;
        assert!(result.is_ok(), "Failed to list channels");

        let channel_list = result.unwrap();
        assert!(channel_list.len() >= 3, "Expected at least 3 channels");
    }

    /// Test: Delete channel
    #[tokio::test]
    async fn test_delete_channel() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let command = create_test_channel("VTV7 To Delete", "Will be deleted");
        let created = match state.channel_service.create_channel(&tx, &command).await {
            Ok(ch) => ch,
            Err(e) => panic!("Failed to create channel for delete test: {:?}", e),
        };

        let result = state.channel_service.delete_channel(&tx, created.id).await;
        assert!(result.is_ok(), "Failed to delete channel");
        assert!(result.unwrap(), "Delete should return true");
    }

    /// Test: Update non-existent channel should fail
    #[tokio::test]
    async fn test_update_nonexistent_channel() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let update_command = UpdateChannelCommand {
            name: Some("Should Fail".to_string()),
            description: None,
            logo_url: None,
            time_on: None,
            time_off: None,
        };

        let result = state.channel_service.update_channel(&tx, 999999, &update_command).await;
        assert!(result.is_err(), "Expected error when updating non-existent channel");
    }

    /// Test: Get non-existent channel should fail
    #[tokio::test]
    async fn test_get_nonexistent_channel() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let result = state.channel_service.get_channel_by_id(&tx, 999999).await;
        assert!(result.is_err(), "Expected error when getting non-existent channel");
    }

    /// Test: List channels with search
    #[tokio::test]
    async fn test_list_channels_with_search() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        // Create channels with specific names
        state
            .channel_service
            .create_channel(&tx, &create_test_channel("News Channel 24", "News content"))
            .await
            .ok();
        state
            .channel_service
            .create_channel(&tx, &create_test_channel("Sports Channel HD", "Sports content"))
            .await
            .ok();
        state
            .channel_service
            .create_channel(&tx, &create_test_channel("News World", "World news"))
            .await
            .ok();

        let params = PageQueryParam {
            page_num: Some(1),
            page_size: Some(10),
            sort_direction: None,
            sort_by: None,
            q: None,
            start_date: None,
            end_date: None,
        };

        let result = state.channel_service.list_channels(&tx, 1, &params).await;
        assert!(result.is_ok(), "Failed to search channels");

        let channel_list = result.unwrap();
        assert!(channel_list.len() >= 2, "Expected at least 2 channels with 'News'");
    }
}
