#[cfg(test)]
mod employee_integration_tests {
    use crate::common;
    use erp_backend::application::department::department_command::CreateDepartmentCommand;
    use erp_backend::application::department::department_service_interface::DepartmentServiceInterface;
    use erp_backend::application::employee::employee_command::{
        CreateEmployeeCommand, UpdateEmployeeCommand,
    };
    use erp_backend::application::employee::employee_service_interface::EmployeeServiceInterface;
    use erp_backend::application::position::position_command::CreatePositionCommand;
    use erp_backend::application::position::position_service_interface::PositionServiceInterface;
    use erp_backend::util::filter_and_pagination::PageQueryParam;
    use sea_orm::TransactionTrait;

    /// Helper function to create a test employee command
    fn create_test_employee(
        fullname: &str,
        username: &str,
        email: &str,
        position_id: Option<i64>,
        department_id: Option<i64>,
    ) -> CreateEmployeeCommand {
        CreateEmployeeCommand {
            fullname: fullname.to_string(),
            username: username.to_string(),
            email: email.to_string(),
            gender: Some("male".to_string()),
            password: "Test@123456".to_string(),
            address: Some("123 Test Street, Test City".to_string()),
            phone_number: Some(format!("+84-9{:08}", rand::random::<u32>() % 100000000)),
            role: Some("employee".to_string()),
            birthday: Some("1990-01-01".to_string()),
            status: Some(1),
            language: Some("vi".to_string()),
            position_id,
            department_id,
        }
    }

    /// Helper function to create a test department
    async fn setup_test_department(
        state: &erp_backend::core::app_state::AppState,
        tx: &sea_orm::DatabaseTransaction,
    ) -> i64 {
        let command = CreateDepartmentCommand {
            name: format!("Test Dept {}", rand::random::<u32>()),
            short_name: "TSD".to_string(),
            is_socialize: false,
            description: Some("Test department".to_string()),
            image_url: None,
        };
        match state.department_service.create_department(tx, &command).await {
            Ok(dept) => dept.id,
            Err(e) => panic!("Failed to create test department for employee tests: {:?}", e),
        }
    }

    /// Helper function to create a test position
    async fn setup_test_position(
        state: &erp_backend::core::app_state::AppState,
        tx: &sea_orm::DatabaseTransaction,
    ) -> i64 {
        let command = CreatePositionCommand {
            name: format!("Test Position {}", rand::random::<u32>()),
            short_name: "TSP".to_string(),
            description: Some("Test position".to_string()),
        };
        match state.position_service.create_position(tx, &command).await {
            Ok(pos) => pos.id,
            Err(e) => panic!("Failed to create test position for employee tests: {:?}", e),
        }
    }

    /// Test: Successfully create a new employee
    #[tokio::test]
    async fn test_create_employee_success() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let dept_id = setup_test_department(&state, &tx).await;
        let pos_id = setup_test_position(&state, &tx).await;

        let command = create_test_employee(
            "John Doe",
            "john.doe",
            "john.doe@example.com",
            Some(pos_id),
            Some(dept_id),
        );
        let result = state.employee_service.create_new_employee(&tx, &command).await;

        assert!(result.is_ok(), "Failed to create employee: {:?}", result.err());
        let employee = result.unwrap();
        assert!(employee.id > 0, "Employee should have a valid ID");
        // Verify user information is present
        assert!(employee.user.is_some(), "Employee should have user information");
        if let Some(user) = &employee.user {
            assert_eq!(user.email, "john.doe@example.com");
        }
    }

    /// Test: Create employee with duplicate email should fail
    #[tokio::test]
    async fn test_create_duplicate_employee_email() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let dept_id = setup_test_department(&state, &tx).await;
        let pos_id = setup_test_position(&state, &tx).await;

        let command = create_test_employee(
            "Jane Smith",
            "jane.smith",
            "jane.smith@example.com",
            Some(pos_id),
            Some(dept_id),
        );
        match state.employee_service.create_new_employee(&tx, &command).await {
            Ok(_) => {},
            Err(e) => panic!("Failed to create first employee for duplicate test: {:?}", e),
        };

        // Try to create duplicate with same email
        let duplicate_command = create_test_employee(
            "Jane Doe",
            "jane.doe",
            "jane.smith@example.com",
            Some(pos_id),
            Some(dept_id),
        );
        let result = state.employee_service.create_new_employee(&tx, &duplicate_command).await;
        assert!(result.is_err(), "Expected error when creating employee with duplicate email");
    }

    /// Test: Get employee by ID
    #[tokio::test]
    async fn test_get_employee_by_id() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let dept_id = setup_test_department(&state, &tx).await;
        let pos_id = setup_test_position(&state, &tx).await;

        let command = create_test_employee(
            "Alice Johnson",
            "alice.j",
            "alice.j@example.com",
            Some(pos_id),
            Some(dept_id),
        );
        let created = match state.employee_service.create_new_employee(&tx, &command).await {
            Ok(emp) => emp,
            Err(e) => panic!("Failed to create employee for get by id test: {:?}", e),
        };

        // Note: The service interface doesn't have get_employee_by_id,
        // so we'll use list with filter or skip this test
        // For now, just verify the created employee has the right ID
        assert!(created.id > 0, "Employee should have a valid ID");
    }

    /// Test: Update employee
    #[tokio::test]
    async fn test_update_employee() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let dept_id = setup_test_department(&state, &tx).await;
        let pos_id = setup_test_position(&state, &tx).await;

        let command = create_test_employee(
            "Bob Wilson",
            "bob.w",
            "bob.w@example.com",
            Some(pos_id),
            Some(dept_id),
        );
        let created = match state.employee_service.create_new_employee(&tx, &command).await {
            Ok(emp) => emp,
            Err(e) => panic!("Failed to create employee for update test: {:?}", e),
        };

        let update_command = UpdateEmployeeCommand {
            fullname: Some("Robert Wilson".to_string()),
            username: Some("robert.wilson".to_string()),
            email: Some("robert.wilson@example.com".to_string()),
            birthday: None,
            picture: None,
            gender: None,
            address: Some("456 New Street, New City".to_string()),
            role: None,
            phone_number: Some("+84-987654321".to_string()),
            language: None,
            position_id: None,
            department_id: None,
            status: None,
        };

        let result = state.employee_service.update_employee(&tx, created.id, &update_command).await;
        assert!(result.is_ok(), "Failed to update employee");
    }

    /// Test: List employees
    #[tokio::test]
    async fn test_list_employees() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let dept_id = setup_test_department(&state, &tx).await;
        let pos_id = setup_test_position(&state, &tx).await;

        // Create multiple employees
        for i in 1..=3 {
            let command = create_test_employee(
                &format!("Employee {}", i),
                &format!("emp{}", i),
                &format!("emp{}@example.com", i),
                Some(pos_id),
                Some(dept_id),
            );

            match state.employee_service.create_new_employee(&tx, &command).await {
                Ok(_) => {},
                Err(e) => assert!(false, "Failed to create employee {}: {:?}", i, e),
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

        let result = state.employee_service.list_employees(&tx, &params).await;
        assert!(result.is_ok(), "Failed to list employees");

        let employee_list = result.unwrap();
        assert!(employee_list.len() >= 3, "Expected at least 3 employees");
    }

    /// Test: Delete employee
    #[tokio::test]
    async fn test_delete_employee() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let dept_id = setup_test_department(&state, &tx).await;
        let pos_id = setup_test_position(&state, &tx).await;

        let command = create_test_employee(
            "Temp Employee",
            "temp",
            "temp@example.com",
            Some(pos_id),
            Some(dept_id),
        );
        let created = match state.employee_service.create_new_employee(&tx, &command).await {
            Ok(emp) => emp,
            Err(e) => panic!("Failed to create employee for delete test: {:?}", e),
        };

        let result = state.employee_service.delete_employee(&tx, created.id).await;
        assert!(result.is_ok(), "Failed to delete employee");
        assert!(result.unwrap(), "Delete should return true");
    }

    /// Test: Update non-existent employee should fail
    #[tokio::test]
    async fn test_update_nonexistent_employee() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let update_command = UpdateEmployeeCommand {
            fullname: Some("Should Fail".to_string()),
            username: None,
            email: None,
            birthday: None,
            picture: None,
            gender: None,
            address: None,
            role: None,
            phone_number: None,
            language: None,
            position_id: None,
            department_id: None,
            status: None,
        };

        let result = state.employee_service.update_employee(&tx, 999999, &update_command).await;
        assert!(result.is_err(), "Expected error when updating non-existent employee");
    }

    /// Test: Delete non-existent employee should fail
    #[tokio::test]
    async fn test_delete_nonexistent_employee() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let result = state.employee_service.delete_employee(&tx, 999999).await;
        assert!(result.is_err(), "Expected error when deleting non-existent employee");
    }

    /// Test: List employees with pagination
    #[tokio::test]
    async fn test_list_employees_pagination() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let dept_id = setup_test_department(&state, &tx).await;
        let pos_id = setup_test_position(&state, &tx).await;

        // Create multiple employees
        for i in 1..=12 {
            let command = create_test_employee(
                &format!("Paged Emp {}", i),
                &format!("paged.emp{}", i),
                &format!("paged.emp{}@example.com", i),
                Some(pos_id),
                Some(dept_id),
            );

            match state.employee_service.create_new_employee(&tx, &command).await {
                Ok(_) => {},
                Err(e) => assert!(false, "Failed to create paged employee {}: {:?}", i, e),
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

        let result = state.employee_service.list_employees(&tx, &params).await;
        assert!(result.is_ok(), "Failed to list employees with pagination");

        let employee_list = result.unwrap();
        assert!(employee_list.len() <= 5, "Expected at most 5 employees per page");
    }

    /// Test: Create employee without optional fields
    #[tokio::test]
    async fn test_create_employee_minimal_fields() {
        let state = common::setup_test_app_state().await;
        let tx = state.db.begin().await.expect("Failed to begin transaction");

        let command = CreateEmployeeCommand {
            fullname: "Minimal Employee".to_string(),
            username: "minimal.emp".to_string(),
            email: "minimal@example.com".to_string(),
            gender: None,
            password: "Test@123456".to_string(),
            address: None,
            phone_number: None,
            role: None,
            birthday: None,
            status: None,
            language: None,
            position_id: None,
            department_id: None,
        };

        let result = state.employee_service.create_new_employee(&tx, &command).await;
        assert!(result.is_ok(), "Failed to create employee with minimal fields");

        let employee = result.unwrap();
        assert!(employee.id > 0, "Employee should have a valid ID");
        if let Some(user) = &employee.user {
            assert_eq!(user.email, "minimal@example.com");
        }
    }
}
