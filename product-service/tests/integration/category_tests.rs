#[cfg(test)]
mod category_integration_tests {
    use crate::common;
    use erp_backend::application::category::category_command::{
        CreateCategoryCommand, ListCategoriesQueryParam, UpdateCategoryCommand,
    };
    use erp_backend::application::category::category_service_interface::CategoryServiceInterface;
    use sea_orm::TransactionTrait;

    /// Helper function to create a test category command
    fn create_test_category(
        name: &str,
        description: &str,
        category_group_id: Option<i64>,
    ) -> CreateCategoryCommand {
        CreateCategoryCommand {
            name: name.to_string(),
            public_name: format!("{} (Public)", name),
            short_name: name[..3.min(name.len())].to_uppercase(),
            description: Some(description.to_string()),
            category_group_id,
            duration: 30,
            owner_team_id: 1,
            production_unit_ids: vec![1],
            channel_id: vec![1],
            need_register_program: true,
            can_be_long_name_program: false,
            time_assign_media: 7,
            can_be_show_name_of_program: true,
            do_have_song: false,
            need_assign_master_file: true,
            can_assign_master_file_later: false,
            need_review_program: true,
            archive_date: 365,
        }
    }

    /// Test: Successfully create a new category
    #[tokio::test]
    async fn test_create_category_success() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let command = create_test_category("News", "News programs", None);
        let result = state.category_service.create_category(&tx, &command).await;

        assert!(result.is_ok(), "Failed to create category: {:?}", result.err());
        let category = result.unwrap();
        assert_eq!(category.name, "News");
        assert_eq!(category.description, "News programs");
    }

    /// Test: Create category with duplicate name should fail
    #[tokio::test]
    async fn test_create_duplicate_category() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let command = create_test_category("Sports", "Sports programming", None);
        match state.category_service.create_category(&tx, &command).await {
            Ok(_) => {},
            Err(e) => panic!("Failed to create first category for duplicate test: {:?}", e),
        };

        // Try to create duplicate
        let result = state.category_service.create_category(&tx, &command).await;
        assert!(result.is_err(), "Expected error when creating duplicate category");
    }

    /// Test: Get category by ID
    #[tokio::test]
    async fn test_get_category_by_id() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let command = create_test_category("Entertainment", "Entertainment shows", None);
        let created = match state.category_service.create_category(&tx, &command).await {
            Ok(cat) => cat,
            Err(e) => panic!("Failed to create category for get by id test: {:?}", e),
        };

        let result = state.category_service.get_category_by_id(&tx, created.id).await;
        assert!(result.is_ok(), "Failed to get category by id");

        let category = result.unwrap();
        assert_eq!(category.id, created.id);
        assert_eq!(category.name, "Entertainment");
    }

    /// Test: Update category
    #[tokio::test]
    async fn test_update_category() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let command = create_test_category("Drama", "Original description", None);
        let created = match state.category_service.create_category(&tx, &command).await {
            Ok(cat) => cat,
            Err(e) => panic!("Failed to create category for update test: {:?}", e),
        };

        let update_command = UpdateCategoryCommand {
            owner_team_id: None,
            production_unit_ids: None,
            channel_ids: None,
            name: Some("Drama Series".to_string()),
            public_name: None,
            short_name: None,
            description: Some("Updated drama series description".to_string()),
            duration: None,
            archive_date: None,
            need_register_program: None,
            can_be_long_name_program: None,
            time_assign_media: None,
            can_be_show_name_of_program: None,
            do_have_song: None,
            need_assign_master_file: None,
            can_assign_master_file_later: None,
            need_review_program: None,
        };

        let result = state.category_service.update_category(&tx, created.id, &update_command).await;
        assert!(result.is_ok(), "Failed to update category");

        let updated = match state.category_service.get_category_by_id(&tx, created.id).await {
            Ok(cat) => cat,
            Err(e) => panic!("Failed to get updated category: {:?}", e),
        };
        assert_eq!(updated.name, "Drama Series");
        assert_eq!(updated.description, "Updated drama series description");
    }

    /// Test: List categories
    #[tokio::test]
    async fn test_list_categories() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        // Create multiple categories
        let categories = vec![
            ("Documentary", "Documentary programs"),
            ("Music", "Music shows"),
            ("Talk Show", "Talk show programs"),
        ];

        for (name, desc) in categories {
            let command = create_test_category(name, desc, None);

            match state.category_service.create_category(&tx, &command).await {
                Ok(_) => {},
                Err(e) => assert!(false, "Failed to create category '{}': {:?}", name, e),
            };
        }

        let params = ListCategoriesQueryParam {
            channel_id: None,
            owner_team_id: None,
            page_num: Some(1),
            page_size: Some(10),
            sort_direction: None,
            sort_by: None,
            q: None,
        };

        let result = state.category_service.list_categories(&tx, &params).await;
        assert!(result.is_ok(), "Failed to list categories");

        let category_list = result.unwrap();
        assert!(category_list.len() >= 3, "Expected at least 3 categories");
    }

    /// Test: Delete category
    #[tokio::test]
    async fn test_delete_category() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let command = create_test_category("Temporary Category", "Will be deleted", None);
        let created = match state.category_service.create_category(&tx, &command).await {
            Ok(cat) => cat,
            Err(e) => panic!("Failed to create category for delete test: {:?}", e),
        };

        let result = state.category_service.delete_category(&tx, created.id).await;
        assert!(result.is_ok(), "Failed to delete category");
        assert!(result.unwrap(), "Delete should return true");
    }

    // SKIPPED: assign_category_to_channel method does not exist in CategoryService
    // /// Test: Assign category to channel
    // #[tokio::test]
    // async fn test_assign_category_to_channel() {
    //     let state = common::setup_test_app_state().await;
    //     let tx = state.db.begin().await.expect("Failed to begin transaction");

    //     // Create a channel
    //     let channel_command = create_test_channel("VTV1");
    //     let channel = state
    //         .channel_service
    //         .create_channel(&tx, &channel_command)
    //         .await
    //         .expect("Failed to create channel");

    //     // Create a category
    //     let category_command = create_test_category("News Programs", "News category", None);
    //     let category = state
    //         .category_service
    //         .create_category(&tx, &category_command)
    //         .await
    //         .expect("Failed to create category");

    //     // Assign category to channel
    //     let result = state
    //         .category_service
    //         .assign_category_to_channel(&tx, category.id, channel.id)
    //         .await;
    //     assert!(result.is_ok(), "Failed to assign category to channel");
    // }

    // SKIPPED: get_categories_by_channel method does not exist in CategoryService
    // /// Test: Get categories by channel
    // #[tokio::test]
    // async fn test_get_categories_by_channel() {
    //     let state = common::setup_test_app_state().await;
    //     let tx = state.db.begin().await.expect("Failed to begin transaction");

    //     // Create a channel
    //     let channel_command = create_test_channel("VTV2");
    //     let channel = state
    //         .channel_service
    //         .create_channel(&tx, &channel_command)
    //         .await
    //         .expect("Failed to create channel");

    //     // Create and assign multiple categories
    //     for i in 1..=3 {
    //         let category_command = create_test_category(
    //             &format!("Category {}", i),
    //             &format!("Test category {}", i),
    //             None,
    //         );
    //         let category = state
    //             .category_service
    //             .create_category(&tx, &category_command)
    //             .await
    //             .expect("Failed to create category");

    //         state
    //             .category_service
    //             .assign_category_to_channel(&tx, category.id, channel.id)
    //             .await
    //             .ok();
    //     }

    //     let result = state
    //         .category_service
    //         .get_categories_by_channel(&tx, channel.id)
    //         .await;
    //     assert!(result.is_ok(), "Failed to get categories by channel");

    //     let categories = result.unwrap();
    //     assert!(categories.len() >= 3, "Expected at least 3 categories for the channel");
    // }

    /// Test: Update non-existent category should fail
    #[tokio::test]
    async fn test_update_nonexistent_category() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let update_command = UpdateCategoryCommand {
            owner_team_id: None,
            production_unit_ids: None,
            channel_ids: None,
            name: Some("Should Fail".to_string()),
            public_name: None,
            short_name: None,
            description: None,
            duration: None,
            archive_date: None,
            need_register_program: None,
            can_be_long_name_program: None,
            time_assign_media: None,
            can_be_show_name_of_program: None,
            do_have_song: None,
            need_assign_master_file: None,
            can_assign_master_file_later: None,
            need_review_program: None,
        };

        let result = state.category_service.update_category(&tx, 999999, &update_command).await;
        assert!(result.is_err(), "Expected error when updating non-existent category");
    }

    /// Test: Get non-existent category should fail
    #[tokio::test]
    async fn test_get_nonexistent_category() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let result = state.category_service.get_category_by_id(&tx, 999999).await;
        assert!(result.is_err(), "Expected error when getting non-existent category");
    }

    /// Test: Delete non-existent category should fail
    #[tokio::test]
    async fn test_delete_nonexistent_category() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let result = state.category_service.delete_category(&tx, 999999).await;
        assert!(result.is_err(), "Expected error when deleting non-existent category");
    }

    /// Test: List categories with pagination
    #[tokio::test]
    async fn test_list_categories_pagination() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        // Create multiple categories
        for i in 1..=12 {
            let command =
                create_test_category(&format!("Cat {}", i), &format!("Category {}", i), None);

            match state.category_service.create_category(&tx, &command).await {
                Ok(_) => {},
                Err(e) => assert!(false, "Failed to create category {}: {:?}", i, e),
            };
        }

        let params = ListCategoriesQueryParam {
            channel_id: None,
            owner_team_id: None,
            page_num: Some(1),
            page_size: Some(5),
            sort_direction: None,
            sort_by: None,
            q: None,
        };

        let result = state.category_service.list_categories(&tx, &params).await;
        assert!(result.is_ok(), "Failed to list categories with pagination");

        let category_list = result.unwrap();
        assert!(category_list.len() <= 5, "Expected at most 5 categories per page");
    }
}
