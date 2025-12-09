# Email Verification API Implementation Guide

## FR-US-002: Email Verification

### Overview
The Email Verification feature allows users to verify their email address using a token sent via email during registration. It also supports resending verification emails with rate limiting to prevent abuse. This implementation follows Clean Architecture principles with proper separation of concerns across domain, application, infrastructure, and presentation layers.

## API Endpoints

### 1. Verify Email

**POST** `/v1/auth/verify-email`

#### Request

##### Headers
```
Content-Type: application/json
```

##### Body (VerifyEmailCommand)
```json
{
  "verification_token": "550e8400-e29b-41d4-a716-446655440000"
}
```

##### Field Validation

| Field | Required | Validation Rules |
|-------|----------|------------------|
| `verification_token` | Yes | - Must exist in database<br>- Must not be expired (24h limit)<br>- User must not be already verified |

#### Response

##### Success (200 OK)
```json
{
  "message": "Email verified successfully. You can now login.",
  "data": true,
  "total": 1
}
```

##### Error Responses

**400 Bad Request** - Invalid or Expired Token
```json
{
  "detail": "Verification token has expired. Please request a new verification email"
}
```

**400 Bad Request** - Already Verified
```json
{
  "detail": "User email is already verified"
}
```

**500 Internal Server Error**
```json
{
  "detail": "Internal server error"
}
```

#### Processing Flow

1. **Application Layer - Database-Dependent Business Rule**
   - `VerificationTokenMustExist`: Finds user by verification token

2. **Domain Layer - Business Rules Validation** (enforced in `ModelEx::verify_email`)
   - `UserMustNotBeAlreadyVerified`: Ensures user isn't already verified
   - `VerificationTokenMustNotBeExpired`: Validates token hasn't expired (24h limit)

3. **Email Verification**
   - Update user status from "pending" to "active"
   - Set email_verified_at timestamp
   - Clear verification_token and verification_token_expiry

4. **Database Persistence**
   - Update user record in database
   - Transaction committed

5. **Event Publishing**
   - Publish `UserActivated` event to Kafka topic: "user_activated"
   - Event payload includes: user_id, email, verified_at

6. **Response**
   - Return success message

---

### 2. Resend Verification Email

**POST** `/v1/auth/resend-verification`

#### Request

##### Headers
```
Content-Type: application/json
```

##### Body (ResendVerificationEmailCommand)
```json
{
  "email": "user@example.com"
}
```

##### Field Validation

| Field | Required | Validation Rules |
|-------|----------|------------------|
| `email` | Yes | - User must exist in database<br>- User must not be already verified<br>- Rate limit: max 3 resends per hour |

#### Response

##### Success (200 OK)
```json
{
  "message": "Verification email has been resent. Please check your inbox.",
  "data": true,
  "total": 1
}
```

##### Error Responses

**400 Bad Request** - Rate Limit Exceeded
```json
{
  "detail": "Verification email resend limit exceeded. Maximum 3 resends allowed per hour"
}
```

**400 Bad Request** - Already Verified
```json
{
  "detail": "User email is already verified"
}
```

**404 Not Found** - User Not Found
```json
{
  "detail": "User with email user@example.com not found"
}
```

**500 Internal Server Error**
```json
{
  "detail": "Internal server error"
}
```

#### Processing Flow

1. **Application Layer - Find User**
   - Find user by email address
   - Return 404 if user doesn't exist

2. **Domain Layer - Business Rules Validation** (enforced in `ModelEx::prepare_resend_verification`)
   - `UserMustNotBeAlreadyVerified`: Ensures user isn't already verified
   - `VerificationResendLimitMustNotBeExceeded`: Enforces max 3 resends per hour
   - **Rate Limit Reset Logic**: Counter resets if more than 1 hour has passed since last resend

3. **Token Regeneration**
   - Generate new UUID-based verification token
   - Set new expiry to 24 hours from now
   - Increment verification_resend_count
   - Update last_verification_resend_at timestamp

4. **Database Persistence**
   - Update user record with new token and counters
   - Transaction committed

5. **Event Publishing**
   - Publish `UserRegistered` event again to Kafka topic: "user_registered"
   - Reuses existing notification flow to send verification email
   - Event payload includes: user_id, email, full_name, new verification_token, current timestamp

6. **Response**
   - Return success message

---

## Architecture Implementation

> **🏗️ Architecture Note**: This implementation follows **Domain-Driven Design (DDD)** principles:
> - **Domain Layer** (`src/domain/user/user.rs`): Business rules that don't require external dependencies (token expiration, already verified check, rate limiting) are enforced in domain methods `verify_email()` and `prepare_resend_verification()`.
> - **Application Layer** (`src/application/user/user_service.rs`): Business rules requiring database access (token existence, user lookup) are validated in the application service before delegating to the domain layer.
> - This ensures the domain layer remains pure and testable without external dependencies.

### Domain Layer
**Location**: `src/domain/user/`

- **Model**: `user.rs`
  - `ModelEx::verify_email()`: Verifies email and activates user
    - **Enforces business rules internally** (UserMustNotBeAlreadyVerified, VerificationTokenMustNotBeExpired)
    - Updates status from PENDING to ACTIVE
    - Sets email_verified_at timestamp
    - Clears verification token and expiry

  - `ModelEx::prepare_resend_verification()`: Prepares user for verification email resend
    - **Enforces business rules internally** (UserMustNotBeAlreadyVerified, VerificationResendLimitMustNotBeExceeded)
    - Implements rate limit reset logic (counter resets after 1 hour)
    - Updates verification token, expiry, resend count, and last resend timestamp

- **Business Rules**: `rules/`
  - **Domain-Level Rules** (no database dependency):
    - `VerificationTokenMustNotBeExpired`: Validates token expiry (24h limit)
    - `UserMustNotBeAlreadyVerified`: Ensures user isn't already verified
    - `VerificationResendLimitMustNotBeExceeded`: Enforces max 3 resends per hour with auto-reset
  - **Application-Level Rules** (require database access):
    - `VerificationTokenMustExist`: Checked in application layer (database query)

- **Events**: `events/`
  - `UserActivatedEvent`: Domain event published when email is verified

- **Verification**: `verification.rs`
  - `generate_verification_token()`: Generates new token and expiry
  - `is_token_expired()`: Checks if token has expired

### Application Layer
**Location**: `src/application/user/`

- **Commands**: `user_command.rs`
  - `VerifyEmailCommand`: Input DTO for email verification
  - `ResendVerificationEmailCommand`: Input DTO for resending verification

- **Service**: `user_service.rs`
  - `UserService::verify_email()`: Orchestrates email verification logic
    - **Validates database-dependent business rule**: VerificationTokenMustExist
    - Delegates to domain layer for verification (which enforces domain rules)
    - Persists updated user to database
    - Publishes UserActivated event to Kafka

  - `UserService::resend_verification_email()`: Orchestrates resend logic
    - Finds user by email (database query)
    - Generates new verification token
    - Delegates to domain layer for resend preparation (which enforces domain rules)
    - Persists updated user to database
    - Publishes UserRegistered event to Kafka (reuses existing notification flow)

- **Service Interface**: `user_service_interface.rs`
  - `UserServiceInterface::verify_email()`: Contract definition
  - `UserServiceInterface::resend_verification_email()`: Contract definition

### Infrastructure Layer
**Location**: `src/infrastructure/`

- **Repository**: `model/user_repository.rs`
  - `find_user_by_verification_token()`: Finds user by verification token
  - `find_user_by_email()`: Finds user by email address
  - `update_user()`: Updates user record
  - Database operations via SeaORM

- **Persistence**: Kafka producer for event publishing

### Presentation Layer
**Location**: `src/presentation/user/`

- **Serializer**: `user.rs`
  - Uses `bool` as response data type for both endpoints

### API Layer
**Location**: `src/api/domain/user/`

- **Controller**: `user.rs`
  - `controller_verify_email()`: HTTP endpoint handler for email verification
  - `controller_resend_verification_email()`: HTTP endpoint handler for resend
  - Handles transactions and error responses

---

## Database Schema Updates

### Migration File
**Location**: `user_migration/src/m20251209_000000_add_email_verification_resend_tracking.rs`

This migration adds support for rate limiting verification email resends.

### Table: `users`

**New Fields Added (ALTER TABLE)**:
```sql
ALTER TABLE users
ADD COLUMN verification_resend_count INTEGER DEFAULT 0 NOT NULL;

ALTER TABLE users
ADD COLUMN last_verification_resend_at TIMESTAMP NULL;
```

**Purpose**:
- `verification_resend_count`: Tracks number of resend requests within current 1-hour window
- `last_verification_resend_at`: Timestamp of last resend request, enables auto-reset logic

**Complete Verification-Related Fields** (from all migrations):
```sql
-- From m20251126_142840_create_user_table.rs
verification_token VARCHAR NULL,
verification_token_expiry TIMESTAMP NULL,
email_verified_at TIMESTAMP NULL,
status VARCHAR(10) DEFAULT 'pending' NOT NULL,

-- From m20251209_000000_add_email_verification_resend_tracking.rs
verification_resend_count INTEGER DEFAULT 0 NOT NULL,
last_verification_resend_at TIMESTAMP NULL
```

### Running Migrations

```bash
# Run all pending migrations
cd user_migration
cargo run -- up

# Or rollback the resend tracking migration
cargo run -- down
```

---

## Kafka Event Schemas

### 1. UserActivated Event

**Topic**: `user_activated`

**Event Payload**:
```json
{
  "user_id": 12345,
  "email": "user@example.com",
  "verified_at": "2025-12-09T10:30:00"
}
```

**Published When**: User successfully verifies their email address

**Consumer Actions**:
- Send welcome email
- Trigger onboarding workflow
- Update analytics/metrics

---

### 2. UserRegistered Event (Resend)

**Topic**: `user_registered`

**Event Payload**:
```json
{
  "user_id": 12345,
  "email": "user@example.com",
  "full_name": "John Doe",
  "verification_token": "new-uuid-token",
  "created_at": "2025-12-09T11:00:00"
}
```

**Published When**: User requests resend of verification email

**Consumer Actions**:
- Send verification email with new token
- Same flow as initial registration

---

## Business Rules Deep Dive

### Rate Limiting Logic

The resend verification feature implements sophisticated rate limiting:

#### Counter Reset Mechanism
```rust
// In ModelEx::prepare_resend_verification()
let now = Utc::now().naive_utc();

// Reset counter if more than 1 hour has passed
if let Some(last_resend) = self.last_verification_resend_at {
    let one_hour_ago = now - Duration::hours(1);
    if last_resend <= one_hour_ago {
        self.verification_resend_count = 0;
    }
}
```

#### Rate Limit Enforcement
- **Max Resends**: 3 per hour
- **Window**: Rolling 1-hour window (not fixed hour)
- **Reset**: Automatic counter reset after 1 hour from last resend
- **Enforcement**: Domain layer business rule `VerificationResendLimitMustNotBeExceeded`

#### Example Scenarios

**Scenario 1: Normal Usage**
```
10:00 AM - User requests resend #1 (count: 1)
10:15 AM - User requests resend #2 (count: 2)
10:30 AM - User requests resend #3 (count: 3)
10:45 AM - User requests resend #4 → ❌ REJECTED (limit exceeded)
11:31 AM - User requests resend #5 → ✅ ALLOWED (1 hour passed, count reset to 1)
```

**Scenario 2: Edge Case**
```
09:00 AM - User requests resend #1 (count: 1)
09:30 AM - User requests resend #2 (count: 2)
10:31 AM - User requests resend #3 → ✅ ALLOWED (count reset to 1, last resend was >1 hour ago)
```

### Token Expiration

- **Expiry Period**: 24 hours from generation
- **Validation**: Enforced in domain layer by `VerificationTokenMustNotBeExpired`
- **Clock**: Uses UTC timezone for consistency
- **Behavior**: Expired tokens are rejected with clear error message

---

## Security Considerations

1. **Token Security**: UUID v4 for verification tokens (128-bit random)
2. **Token Expiry**: 24-hour expiration for security
3. **Rate Limiting**: Prevents email bombing/abuse (max 3 resends/hour)
4. **Automatic Reset**: Counter auto-resets after 1 hour (user-friendly)
5. **Transaction Safety**: Database rollback on errors
6. **Status Validation**: Cannot verify already-verified users
7. **Token Invalidation**: Tokens cleared after successful verification

---

## Testing

### Sample cURL Requests

#### 1. Verify Email
```bash
curl -X POST http://localhost:8080/v1/auth/verify-email \
  -H "Content-Type: application/json" \
  -d '{
    "verification_token": "550e8400-e29b-41d4-a716-446655440000"
  }'
```

#### 2. Resend Verification Email
```bash
curl -X POST http://localhost:8080/v1/auth/resend-verification \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com"
  }'
```

### Expected Workflow

#### Happy Path: Email Verification
1. User receives verification email after registration
2. User clicks verification link (contains token)
3. Frontend calls POST `/v1/auth/verify-email` with token
4. API validates token exists and hasn't expired
5. Domain layer validates user not already verified
6. User status updated from "pending" to "active"
7. `UserActivated` event published to Kafka
8. User can now login

#### Happy Path: Resend Verification
1. User didn't receive email or token expired
2. User requests new verification email
3. Frontend calls POST `/v1/auth/resend-verification` with email
4. API finds user by email
5. Domain layer validates not already verified and rate limit
6. New token generated (24h expiry)
7. Resend counter incremented
8. `UserRegistered` event published to Kafka
9. Notification service sends new verification email

#### Error Path: Rate Limit Exceeded
1. User requests resend 3 times within 1 hour
2. User attempts 4th resend within same hour
3. API returns 400 Bad Request with rate limit error
4. User must wait until 1 hour after first resend
5. Counter auto-resets after 1 hour

---

## Integration Points

### 1. Notification Service
- **Consumes**: `user_registered` and `user_activated` events from Kafka
- **Sends**: Verification emails with token link
- **Template Variables**: user_id, email, full_name, verification_token

### 2. Frontend Application
- **Registration Flow**: Display message to check email after registration
- **Verification Link**: `https://app.example.com/verify-email?token={verification_token}`
- **Resend Button**: Allow users to request new verification email
- **Error Handling**: Display appropriate errors (expired token, rate limit, etc.)

### 3. Analytics/Metrics
- **Tracks**: Verification completion rate
- **Monitors**: Resend frequency (detect UX issues)
- **Alerts**: High verification failure rates

---

## Monitoring and Observability

### Key Metrics to Track
1. **Verification Success Rate**: % of users who verify email
2. **Time to Verify**: Average time between registration and verification
3. **Resend Frequency**: Average resends per user
4. **Token Expiry Rate**: % of tokens that expire before use
5. **Rate Limit Hit Rate**: % of users hitting resend limit

### Log Messages
```
INFO - Verifying email with token: xxx
INFO - Email verified successfully
INFO - Resending verification email for: user@example.com
INFO - Verification email resent successfully
ERROR - Failed to verify email: Token expired
ERROR - Failed to resend verification email: Rate limit exceeded
```

---

## Troubleshooting

### Common Issues

**Issue**: "Verification token has expired"
- **Cause**: User waited >24 hours to verify
- **Solution**: Use resend verification endpoint to get new token

**Issue**: "Verification email resend limit exceeded"
- **Cause**: User requested >3 resends within 1 hour
- **Solution**: Wait for counter to auto-reset (1 hour after first resend)

**Issue**: "User email is already verified"
- **Cause**: User already verified or trying to verify twice
- **Solution**: User can proceed to login

**Issue**: "Invalid verification token"
- **Cause**: Token doesn't exist in database (typo, wrong token, manual database change)
- **Solution**: Use resend verification endpoint

---

## Future Enhancements
- Configurable token expiry duration
- Configurable rate limit settings
- SMS verification as alternative to email
- Magic link login (passwordless)
- Email change verification flow
