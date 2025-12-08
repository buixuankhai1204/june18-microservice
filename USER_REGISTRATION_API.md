# User Registration API Implementation Guide

## FR-US-001: User Registration

### Overview
The User Registration feature allows new users to create an account in the system. This implementation follows Clean Architecture principles with proper separation of concerns across domain, application, infrastructure, and presentation layers.

### API Endpoint

**POST** `/v1/auth/register`

### Request

#### Headers
```
Content-Type: application/json
```

#### Body (RegisterUserCommand)
```json
{
  "email": "user@example.com",
  "password": "SecurePass123!",
  "full_name": "John Doe",
  "phone_number": "+1234567890",  // Optional
  "date_of_birth": "2000-01-15"    // Optional
}
```

#### Field Validation

| Field | Required | Validation Rules |
|-------|----------|------------------|
| `email` | Yes | - Must be unique<br>- Valid email format (RFC 5322) |
| `password` | Yes | - Minimum 8 characters<br>- Must contain uppercase letter<br>- Must contain lowercase letter<br>- Must contain number<br>- Must contain special character |
| `full_name` | Yes | - Maximum 100 characters<br>- Cannot be empty |
| `phone_number` | No | - Must be unique if provided<br>- Valid international format (+[1-9][0-9]{1,14}) |
| `date_of_birth` | No | - User must be 13+ years old |

### Response

#### Success (201 Created)
```json
{
  "message": "User registered successfully.",
  "data": {
    "user_id": "12345",
    "email": "user@example.com",
    "message": "Please check your email to verify account"
  },
  "total": 1
}
```

#### Error Responses

**400 Bad Request** - Validation Failed
```json
{
  "detail": "Password must contain at least one uppercase letter"
}
```

**409 Conflict** - Email or Phone Already Exists
```json
{
  "detail": "Email user@example.com already exists in the system"
}
```

**500 Internal Server Error**
```json
{
  "detail": "Internal server error"
}
```

### Processing Flow

1. **Input Validation**
   - Email format validation
   - Password complexity validation
   - Full name length validation
   - Phone number format validation (if provided)
   - Age validation (if date_of_birth provided)

2. **Business Rules Validation**
   - `EmailMustBeValid`: Validates email format using regex
   - `EmailMustBeUnique`: Checks email uniqueness in database
   - `PasswordMustMeetRequirements`: Validates password complexity
   - `FullNameMustBeValid`: Validates full name length and format
   - `PhoneMustBeValid`: Validates phone format (if provided)
   - `PhoneMustBeUnique`: Checks phone uniqueness (if provided)
   - `UserMustBeAtLeastAge`: Validates minimum age of 13 years

3. **User Creation**
   - Parse full_name into first_name and last_name
   - Generate username from email (part before @)
   - Set default role: "customer"
   - Set default status: "pending"
   - Hash password using Argon2 (10 salt rounds equivalent)

4. **Verification Token Generation**
   - Generate UUID-based verification token
   - Set expiry to 24 hours from creation

5. **Database Persistence**
   - Create user record in database
   - Store verification token and expiry

6. **Event Publishing**
   - Publish `UserRegistered` event to Kafka topic: "user_registered"
   - Event payload includes: user_id, email, full_name, verification_token, created_at

7. **Response**
   - Return user_id, email, and verification message

### Architecture Implementation

#### Domain Layer
**Location**: `src/domain/user/`

- **Model**: `user.rs`
  - `ModelEx::create_user_for_registration()`: Creates user domain model
  - Enums: `Status`, `Role`

- **Business Rules**: `rules/`
  - `EmailMustBeValid`
  - `EmailMustBeUnique`
  - `PasswordMustMeetRequirements`
  - `FullNameMustBeValid`
  - `PhoneMustBeValid`
  - `PhoneMustBeUnique`
  - `UserMustBeAtLeastAge`

- **Events**: `events/`
  - `UserRegisteredEvent`: Domain event for user registration

- **Verification**: `verification.rs`
  - `generate_verification_token()`: Generates token and expiry

#### Application Layer
**Location**: `src/application/user/`

- **Command**: `user_command.rs`
  - `RegisterUserCommand`: Input DTO with validation attributes

- **Service**: `user_service.rs`
  - `UserService::register_user()`: Orchestrates registration logic
  - Validates all business rules
  - Coordinates domain, infrastructure, and external services

- **Service Interface**: `user_service_interface.rs`
  - `UserServiceInterface::register_user()`: Contract definition

#### Infrastructure Layer
**Location**: `src/infrastructure/`

- **Repository**: `model/user_repository.rs`
  - `email_exists()`: Checks email uniqueness
  - `phone_exists()`: Checks phone uniqueness
  - Database operations via SeaORM

- **Persistence**: Kafka producer for event publishing

#### Presentation Layer
**Location**: `src/presentation/user/`

- **Serializer**: `user.rs`
  - `UserCreatedSerializer`: Response DTO

#### API Layer
**Location**: `src/api/domain/user/`

- **Controller**: `user.rs`
  - `controller_register_user()`: HTTP endpoint handler
  - Handles transactions and error responses

### Database Schema

**Table**: `users`

```sql
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    avatar VARCHAR NULL,
    first_name VARCHAR NOT NULL,
    last_name VARCHAR NOT NULL,
    username VARCHAR UNIQUE NOT NULL,
    email VARCHAR UNIQUE NOT NULL,
    password VARCHAR NULL,
    birth_of_date DATE NULL,
    phone_number VARCHAR UNIQUE NULL,
    status VARCHAR(10) DEFAULT 'pending' NOT NULL,
    role VARCHAR(20) DEFAULT 'customer' NOT NULL,
    is_deleted BOOLEAN DEFAULT FALSE NOT NULL,
    verification_token VARCHAR NULL,
    verification_token_expiry TIMESTAMP NULL,
    email_verified_at TIMESTAMP NULL,
    created_at TIMESTAMP NULL,
    updated_at TIMESTAMP NULL,
    deleted_at TIMESTAMP NULL
);
```

### Kafka Event Schema

**Topic**: `user_registered`

**Event Payload**:
```json
{
  "user_id": 12345,
  "email": "user@example.com",
  "full_name": "John Doe",
  "verification_token": "550e8400-e29b-41d4-a716-446655440000",
  "created_at": "2025-12-08T10:30:00"
}
```

### Security Considerations

1. **Password Hashing**: Argon2 algorithm (equivalent to 10 bcrypt salt rounds)
2. **Token Security**: UUID v4 for verification tokens
3. **Token Expiry**: 24-hour expiration for email verification
4. **Input Validation**: Multiple layers of validation (presentation, application, domain)
5. **Transaction Safety**: Database rollback on errors

### Testing

#### Sample cURL Request
```bash
curl -X POST http://localhost:8080/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "newuser@example.com",
    "password": "SecurePass123!",
    "full_name": "Jane Smith",
    "phone_number": "+1234567890",
    "date_of_birth": "2000-05-20"
  }'
```

#### Expected Workflow
1. User submits registration form
2. API validates input and business rules
3. Password is hashed
4. User record created with status="pending"
5. Verification token generated (24h expiry)
6. `UserRegistered` event published to Kafka
7. Notification service (consuming Kafka event) sends verification email
8. User verifies email via token
9. User status updated to "active"

### Future Enhancements
- Email verification endpoint implementation
- Resend verification email endpoint
- Social login integration
- Two-factor authentication support
