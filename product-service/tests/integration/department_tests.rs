#[cfg(test)]
mod department_integration_tests {
    use crate::common;
    use erp_backend::application::department::department_command::{
        CreateDepartmentCommand, UpdateDepartmentCommand,
    };
    use erp_backend::application::department::department_service_interface::DepartmentServiceInterface;
    use erp_backend::util::filter_and_pagination::PageQueryParam;
    use sea_orm::TransactionTrait;

    /// Helper function to create a test department command
    fn create_test_department(name: &str, description: &str) -> CreateDepartmentCommand {
        CreateDepartmentCommand {
            name: name.to_string(),
            short_name: format!("{}_SHORT", &name[..3.min(name.len())].to_uppercase()),
            is_socialize: false,
            description: Some(description.to_string()),
            image_url: None,
        }
    }

    /// Test: Successfully create a new department
    #[tokio::test]
    async fn test_create_department_success() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let command = create_test_department("IT Department", "Information Technology Department");
        let result = state.department_service.create_department(&tx, &command).await;

        assert!(result.is_ok(), "Failed to create department: {:?}", result.err());
        let department = result.unwrap();
        assert_eq!(department.name, "IT Department");
        assert_eq!(department.description, Some("Information Technology Department".to_string()));
    }

    /// Test: Create department with duplicate name should fail
    #[tokio::test]
    async fn test_create_duplicate_department() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let command = create_test_department("HR Department", "Human Resources");

        match state.department_service.create_department(&tx, &command).await {
            Ok(_) => {},
            Err(e) => panic!("Failed to create first department: {:?}", e),
        };

        // Try to create duplicate
        let result = state.department_service.create_department(&tx, &command).await;
        assert!(result.is_err(), "Expected error when creating duplicate department");
    }

    /// Test: Get department by ID
    #[tokio::test]
    async fn test_get_department_by_id() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let command = create_test_department("Finance Department", "Financial Management");
        let created = match state.department_service.create_department(&tx, &command).await {
            Ok(dept) => dept,
            Err(e) => panic!("Failed to create department for get test: {:?}", e),
        };

        let result = state.department_service.get_department_by_id(&tx, created.id).await;
        assert!(result.is_ok(), "Failed to get department by id");

        let department = result.unwrap();
        assert_eq!(department.id, created.id);
        assert_eq!(department.name, "Finance Department");
    }

    /// Test: Update department
    #[tokio::test]
    async fn test_update_department() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let command = create_test_department("Marketing Dept", "Original description");
        let created = match state.department_service.create_department(&tx, &command).await {
            Ok(dept) => dept,
            Err(e) => panic!("Failed to create department for update test: {:?}", e),
        };

        let update_command = UpdateDepartmentCommand {
            name: Some("Marketing Department".to_string()),
            short_name: None,
            description: Some("Updated marketing department description".to_string()),
            image_url: None,
            is_socialize: None,
            status: None,
        };

        let result =
            state.department_service.update_department(&tx, created.id, &update_command).await;
        assert!(result.is_ok(), "Failed to update department");

        let updated = match state.department_service.get_department_by_id(&tx, created.id).await {
            Ok(dept) => dept,
            Err(e) => panic!("Failed to get updated department: {:?}", e),
        };
        assert_eq!(updated.name, "Marketing Department");
        assert_eq!(
            updated.description,
            Some("Updated marketing department description".to_string())
        );
    }

    /// Test: List departments
    #[tokio::test]
    async fn test_list_departments() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        // Create multiple departments
        for i in 1..=3 {
            let command = create_test_department(
                &format!("Department {}", i),
                &format!("Test department {}", i),
            );

            match state.department_service.create_department(&tx, &command).await {
                Ok(_) => {},
                Err(e) => assert!(false, "Failed to create department {}: {:?}", i, e),
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

        let result = state.department_service.list_departments(&tx, 1, &params).await;
        assert!(result.is_ok(), "Failed to list departments");

        let department_list = result.unwrap();
        assert!(department_list.len() >= 3, "Expected at least 3 departments");
    }

    /// Test: Delete department
    #[tokio::test]
    async fn test_delete_department() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let command = create_test_department("Temp Department", "Will be deleted");
        let created = match state.department_service.create_department(&tx, &command).await {
            Ok(dept) => dept,
            Err(e) => panic!("Failed to create department for delete test: {:?}", e),
        };

        let result = state.department_service.delete_department(&tx, created.id).await;
        assert!(result.is_ok(), "Failed to delete department");
        assert!(result.unwrap(), "Delete should return true");
    }

    /// Test: Update non-existent department should fail
    #[tokio::test]
    async fn test_update_nonexistent_department() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let update_command = UpdateDepartmentCommand {
            name: Some("Should Fail".to_string()),
            short_name: None,
            description: None,
            image_url: None,
            is_socialize: None,
            status: None,
        };

        let result = state.department_service.update_department(&tx, 999999, &update_command).await;
        assert!(result.is_err(), "Expected error when updating non-existent department");
    }

    /// Test: Get non-existent department should fail
    #[tokio::test]
    async fn test_get_nonexistent_department() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let result = state.department_service.get_department_by_id(&tx, 999999).await;
        assert!(result.is_err(), "Expected error when getting non-existent department");
    }

    /// Test: Delete non-existent department should fail
    #[tokio::test]
    async fn test_delete_nonexistent_department() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let result = state.department_service.delete_department(&tx, 999999).await;
        assert!(result.is_err(), "Expected error when deleting non-existent department");
    }

    /// Test: List departments with pagination
    #[tokio::test]
    async fn test_list_departments_pagination() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        // Create multiple departments
        for i in 1..=15 {
            let command = create_test_department(
                &format!("Paginated Dept {}", i),
                &format!("Department number {}", i),
            );

            match state.department_service.create_department(&tx, &command).await {
                Ok(_) => {},
                Err(e) => assert!(false, "Failed to create paginated department {}: {:?}", i, e),
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

        let result = state.department_service.list_departments(&tx, 1, &params).await;
        assert!(result.is_ok(), "Failed to list departments with pagination");

        let department_list = result.unwrap();
        assert!(department_list.len() <= 5, "Expected at most 5 departments per page");
    }
}
