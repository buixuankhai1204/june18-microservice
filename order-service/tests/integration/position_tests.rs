#[cfg(test)]
mod position_integration_tests {
    use crate::common;
    use erp_backend::application::position::position_command::{
        CreatePositionCommand, UpdatePositionCommand,
    };
    use erp_backend::application::position::position_service_interface::PositionServiceInterface;
    use erp_backend::util::filter_and_pagination::PageQueryParam;
    use sea_orm::TransactionTrait;

    /// Helper function to create a test position command
    fn create_test_position(name: &str, description: &str) -> CreatePositionCommand {
        CreatePositionCommand {
            name: name.to_string(),
            short_name: format!("{}_SN", &name[..3.min(name.len())].to_uppercase()),
            description: Some(description.to_string()),
        }
    }

    /// Test: Successfully create a new position
    #[tokio::test]
    async fn test_create_position_success() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let command = create_test_position("Software Engineer", "Develops software applications");
        let result = state.position_service.create_position(&tx, &command).await;

        assert!(result.is_ok(), "Failed to create position: {:?}", result.err());
        let position = result.unwrap();
        assert_eq!(position.name, "Software Engineer");
        assert_eq!(position.description, Some("Develops software applications".to_string()));
    }

    /// Test: Create position with duplicate name should fail
    #[tokio::test]
    async fn test_create_duplicate_position() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let command = create_test_position("Product Manager", "Manages product development");
        match state.position_service.create_position(&tx, &command).await {
            Ok(_) => {},
            Err(e) => panic!("Failed to create first position for duplicate test: {:?}", e),
        };

        // Try to create duplicate
        let result = state.position_service.create_position(&tx, &command).await;
        assert!(result.is_err(), "Expected error when creating duplicate position");
    }

    /// Test: Get position by ID
    #[tokio::test]
    async fn test_get_position_by_id() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let command = create_test_position("Data Analyst", "Analyzes data and creates reports");
        let created = match state.position_service.create_position(&tx, &command).await {
            Ok(pos) => pos,
            Err(e) => panic!("Failed to create position for get by id test: {:?}", e),
        };

        let result = state.position_service.get_position_by_id(&tx, created.id).await;
        assert!(result.is_ok(), "Failed to get position by id");

        let position = result.unwrap();
        assert_eq!(position.id, created.id);
        assert_eq!(position.name, "Data Analyst");
    }

    /// Test: Update position
    #[tokio::test]
    async fn test_update_position() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let command = create_test_position("Designer", "Original description");
        let created = match state.position_service.create_position(&tx, &command).await {
            Ok(pos) => pos,
            Err(e) => panic!("Failed to create position for update test: {:?}", e),
        };

        let update_command = UpdatePositionCommand {
            name: Some("Senior UX Designer".to_string()),
            description: Some("Leads UX design initiatives".to_string()),
            short_name: None,
        };

        let result = state.position_service.update_position(&tx, created.id, &update_command).await;
        assert!(result.is_ok(), "Failed to update position");

        let updated = match state.position_service.get_position_by_id(&tx, created.id).await {
            Ok(pos) => pos,
            Err(e) => panic!("Failed to get updated position: {:?}", e),
        };
        assert_eq!(updated.name, "Senior UX Designer");
        assert_eq!(updated.description, Some("Leads UX design initiatives".to_string()));
    }

    /// Test: List positions
    #[tokio::test]
    async fn test_list_positions() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        // Create multiple positions
        let positions = vec![
            ("Developer", "Software development"),
            ("Tester", "Quality assurance"),
            ("DevOps", "Infrastructure management"),
        ];

        for (i, (name, desc)) in positions.iter().enumerate() {
            let command = create_test_position(name, desc);
            match state.position_service.create_position(&tx, &command).await {
                Ok(_) => {},
                Err(e) => assert!(false, "Failed to create position {} ({}): {:?}", i, name, e),
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

        let result = state.position_service.list_positions(&tx, 1, &params).await;
        assert!(result.is_ok(), "Failed to list positions");

        let position_list = result.unwrap();
        assert!(position_list.len() >= 3, "Expected at least 3 positions");
    }

    /// Test: Delete position
    #[tokio::test]
    async fn test_delete_position() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let command = create_test_position("Temporary Role", "Will be deleted");
        let created = match state.position_service.create_position(&tx, &command).await {
            Ok(pos) => pos,
            Err(e) => panic!("Failed to create position for delete test: {:?}", e),
        };

        let result = state.position_service.delete_position(&tx, created.id).await;
        assert!(result.is_ok(), "Failed to delete position");
        assert!(result.unwrap(), "Delete should return true");
    }

    /// Test: Update non-existent position should fail
    #[tokio::test]
    async fn test_update_nonexistent_position() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let update_command = UpdatePositionCommand {
            name: Some("Should Fail".to_string()),
            description: None,
            short_name: None,
        };

        let result = state.position_service.update_position(&tx, 999999, &update_command).await;
        assert!(result.is_err(), "Expected error when updating non-existent position");
    }

    /// Test: Get non-existent position should fail
    #[tokio::test]
    async fn test_get_nonexistent_position() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let result = state.position_service.get_position_by_id(&tx, 999999).await;
        assert!(result.is_err(), "Expected error when getting non-existent position");
    }

    /// Test: Delete non-existent position should fail
    #[tokio::test]
    async fn test_delete_nonexistent_position() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let result = state.position_service.delete_position(&tx, 999999).await;
        assert!(result.is_err(), "Expected error when deleting non-existent position");
    }

    /// Test: List positions with pagination
    #[tokio::test]
    async fn test_list_positions_pagination() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        // Create multiple positions
        for i in 1..=12 {
            let command =
                create_test_position(&format!("Position {}", i), &format!("Test position {}", i));

            match state.position_service.create_position(&tx, &command).await {
                Ok(_) => {},
                Err(e) => assert!(false, "Failed to create position {}: {:?}", i, e),
            };
        }

        let params = PageQueryParam {
            page_num: Some(1),
            page_size: Some(5),
            sort_direction: None,
            sort_by: None,
            q: None,
            start_date: None,
            end_date: None,
        };

        let result = state.position_service.list_positions(&tx, 1, &params).await;
        assert!(result.is_ok(), "Failed to list positions with pagination");

        let position_list = result.unwrap();
        assert!(position_list.len() <= 5, "Expected at most 5 positions per page");
    }
}
