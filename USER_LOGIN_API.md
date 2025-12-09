# User Login API Implementation Guide

## FR-US-003: User Login

### Overview
The User Login feature authenticates users and issues JWT access/refresh tokens. It includes robust security measures such as failed login attempt tracking, account locking, and device information logging. This implementation follows Clean Architecture principles with proper separation of concerns across domain, application, infrastructure, and presentation layers.

## API Endpoint

**POST** `/v1/login_by_email`

### Request

#### Headers
```
Content-Type: application/json
```

#### Body (LoginByEmailCommand)
```json
{
  "email": "user@example.com",
  "password": "SecurePass123!",
  "device_info": {
    "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)...",
    "ip_address": "192.168.1.100"
  }
}
```

#### Field Validation

| Field | Required | Validation Rules |
|-------|----------|------------------|
| `email` | Yes | - Must be valid email format<br>- Account must exist<br>- Account must be active (email verified) |
| `password` | Yes | - Minimum 8 characters |
| `device_info` | No | - Optional tracking information |
| `device_info.user_agent` | No | - Browser/client user agent string |
| `device_info.ip_address` | No | - Client IP address |

### Response

#### Success (200 OK)
```json
{
  "type": "Token",
  "Token": {
    "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
    "expires_in": 900,
    "user": {
      "id": "12345",
      "email": "user@example.com",
      "full_name": "John Doe",
      "role": "customer"
    }
  }
}
```

#### Error Responses

**401 Unauthorized** - Invalid Credentials
```json
{
  "detail": "Invalid email or password"
}
```

**401 Unauthorized** - Account Not Active
```json
{
  "detail": "Account is not active. Please verify your email or contact support."
}
```

**423 Locked** - Account Temporarily Locked
```json
{
  "detail": "Account is temporarily locked due to too many failed login attempts. Please try again in 25 minutes."
}
```

**423 Locked** - Too Many Failed Attempts
```json
{
  "detail": "Too many failed login attempts. Account will be locked for 30 minutes."
}
```

**500 Internal Server Error**
```json
{
  "detail": "Internal server error"
}
```

### Processing Flow

1. **Application Layer - Find User**
   - Find user by email (database query)
   - Return generic "Invalid email or password" if not found (security: don't reveal if email exists)

2. **Domain Layer - Validate Login Attempt** (enforced in `ModelEx::validate_login_attempt`)
   - `AccountMustNotBeLocked`: Checks if account is currently locked (account_locked_until > now)
   - `AccountMustBeActive`: Validates user status is 'active' (not 'pending' or 'inactive')
   - `FailedLoginLimitMustNotBeExceeded`: Enforces max 5 failed attempts within 15 minutes

3. **Password Verification**
   - Verify password using Argon2
   - If invalid: Handle failed login attempt (domain layer)
   - If valid: Handle successful login (domain layer)

4. **Failed Login Handling** (`ModelEx::handle_failed_login`)
   - Auto-reset counter if >15 minutes passed since last failed attempt
   - Increment failed_login_attempts counter
   - Update last_failed_login_at timestamp
   - Lock account for 30 minutes if failed_login_attempts >= 5

5. **Successful Login Handling** (`ModelEx::handle_successful_login`)
   - Reset failed_login_attempts to 0
   - Clear last_failed_login_at
   - Clear account_locked_until
   - Update last_login_at timestamp

6. **Session Management**
   - Generate unique session ID (UUID v4)
   - Store refresh token in Redis with 7-day expiry
   - Key format: `refresh_token:session:{session_id}`

7. **JWT Token Generation**
   - Access token: 15 minutes expiry (RS256 algorithm)
   - Refresh token: 7 days expiry (RS256 algorithm)
   - Both tokens contain: user_id, session_id, iat, exp

8. **Event Publishing**
   - Publish `UserLoggedIn` event to Kafka topic: "user_logged_in"
   - Event payload includes: user_id, email, session_id, device_info, logged_in_at

9. **Response**
   - Return access_token, refresh_token, expires_in (900 seconds), and user info

---

## Architecture Implementation

> **🏗️ Architecture Note**: This implementation follows **Domain-Driven Design (DDD)** principles:
> - **Domain Layer** (`src/domain/user/user.rs`): Business rules for login validation (account status, lock status, failed attempt limits) are enforced in domain methods.
> - **Application Layer** (`src/application/authen/authen_service.rs`): Orchestrates infrastructure operations (database queries, Redis sessions, Kafka events) and delegates validation to domain layer.
> - This ensures the domain layer remains pure and testable without external dependencies.

### Domain Layer
**Location**: `src/domain/user/`

- **Model**: `user.rs`
  - `ModelEx::validate_login_attempt()`: Validates login preconditions
    - **Enforces business rules internally** (AccountMustNotBeLocked, AccountMustBeActive, FailedLoginLimitMustNotBeExceeded)
    - Returns error if account is locked or inactive
    - Checks failed login count within 15-minute window

  - `ModelEx::handle_failed_login()`: Handles failed login attempt
    - **Auto-reset logic**: Resets counter if >15 minutes passed since last failed attempt
    - Increments failed_login_attempts
    - Updates last_failed_login_at
    - Locks account for 30 minutes if attempts >= 5

  - `ModelEx::handle_successful_login()`: Handles successful login
    - Resets all failed login tracking fields
    - Updates last_login_at timestamp

- **Business Rules**: `rules/`
  - **Domain-Level Rules** (no database dependency):
    - `AccountMustNotBeLocked`: Validates account_locked_until < now
    - `AccountMustBeActive`: Validates status == 'active'
    - `FailedLoginLimitMustNotBeExceeded`: Validates failed attempts < 5 within 15-minute window

- **Events**: `events/`
  - `UserLoggedInEvent`: Domain event published on successful login
  - `DeviceInfoEvent`: Nested event for device tracking

### Application Layer
**Location**: `src/application/authen/`

- **Command**: `authen_command.rs`
  - `LoginByEmailCommand`: Input DTO with email, password, device_info
  - `DeviceInfo`: Optional device tracking information

- **Service**: `authen_service.rs`
  - `AuthenService::login_by_email()`: Orchestrates login logic
    - Finds user by email (database query)
    - Delegates validation to domain layer
    - Verifies password using Argon2
    - Handles failed/successful login via domain methods
    - Generates session ID and stores in Redis
    - Generates JWT tokens
    - Publishes UserLoggedIn event to Kafka

- **Service Interface**: `authen_service_interface.rs`
  - `AuthenServiceInterface::login_by_email()`: Contract definition

### Infrastructure Layer
**Location**: `src/infrastructure/`

- **Repository**: `model/user_repository.rs`
  - `find_user_by_email()`: Finds user by email address
  - `update_user()`: Updates user record (for failed login tracking)

- **Redis**: Session and token storage
  - Key: `refresh_token:session:{session_id}`
  - TTL: 7 days (604800 seconds)

- **Kafka**: Event publishing
  - Topic: "user_logged_in"
  - Producer timeout: 5 seconds

- **JWT**: `application/authen/claim.rs`
  - `service_generate_tokens()`: Generates access and refresh tokens
  - Algorithm: RS256 (asymmetric encryption)
  - Access token expiry: 15 minutes (900 seconds)
  - Refresh token expiry: 7 days

### Presentation Layer
**Location**: `src/presentation/authen/`

- **Serializer**: `authen.rs`
  - `TokenResponse`: Response DTO with tokens and user info
  - `UserInfo`: Nested user information (id, email, full_name, role)
  - `LoginResponse`: Enum wrapper for Token response

### API Layer
**Location**: `src/api/domain/auth/`

- **Controller**: `auth.rs`
  - `controller_login_by_email()`: HTTP endpoint handler
  - Validates command
  - Handles transactions and error responses
  - Returns 200 OK or appropriate error status

---

## Database Schema Updates

### Migration File
**Location**: `user_migration/src/m20251209_000001_add_login_tracking_fields.rs`

This migration adds support for failed login attempt tracking and account locking.

### Table: `users`

**New Fields Added (ALTER TABLE)**:
```sql
ALTER TABLE users
ADD COLUMN failed_login_attempts INTEGER DEFAULT 0 NOT NULL;

ALTER TABLE users
ADD COLUMN last_failed_login_at TIMESTAMP NULL;

ALTER TABLE users
ADD COLUMN account_locked_until TIMESTAMP NULL;

ALTER TABLE users
ADD COLUMN last_login_at TIMESTAMP NULL;
```

**Purpose**:
- `failed_login_attempts`: Counter for failed login attempts within 15-minute window
- `last_failed_login_at`: Timestamp of last failed attempt, enables auto-reset logic
- `account_locked_until`: Timestamp until which account is locked (NULL if not locked)
- `last_login_at`: Timestamp of last successful login

### Running Migrations

```bash
# Run all pending migrations
cd user_migration
cargo run -- up

# Or rollback the login tracking migration
cargo run -- down
```

---

## Kafka Event Schema

### UserLoggedIn Event

**Topic**: `user_logged_in`

**Event Payload**:
```json
{
  "user_id": 12345,
  "email": "user@example.com",
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "device_info": {
    "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
    "ip_address": "192.168.1.100"
  },
  "logged_in_at": "2025-12-09T10:30:00"
}
```

**Published When**: User successfully logs in

**Consumer Actions**:
- Log authentication events for security auditing
- Track user activity patterns
- Update analytics/metrics (DAU, login frequency)
- Trigger fraud detection algorithms
- Send login notification emails (if enabled)

---

## Business Rules Deep Dive

### Failed Login Attempt Tracking

The login feature implements sophisticated failed login tracking with auto-reset:

#### Counter Reset Mechanism
```rust
// In ModelEx::handle_failed_login()
let now = Utc::now().naive_utc();

// Reset counter if more than 15 minutes passed since last failed attempt
if let Some(last_failed) = self.last_failed_login_at {
    let fifteen_minutes_ago = now - Duration::minutes(15);
    if last_failed <= fifteen_minutes_ago {
        self.failed_login_attempts = 0;
    }
}
```

#### Account Locking Logic
- **Threshold**: 5 failed attempts within 15 minutes
- **Lockout Duration**: 30 minutes from 5th failed attempt
- **Reset**: Automatic counter reset after 15 minutes from last failed attempt
- **Unlock**: Automatic unlock after 30 minutes (checked on next login attempt)

#### Example Scenarios

**Scenario 1: Normal Failed Attempts**
```
10:00 AM - Failed attempt #1 (count: 1)
10:05 AM - Failed attempt #2 (count: 2)
10:10 AM - Failed attempt #3 (count: 3)
10:15 AM - Failed attempt #4 (count: 4)
10:20 AM - Failed attempt #5 (count: 5, account locked until 10:50 AM)
10:25 AM - Login attempt → ❌ REJECTED (account locked, 25 minutes remaining)
10:51 AM - Login attempt → ✅ ALLOWED (lock expired, validated normally)
```

**Scenario 2: Auto-Reset Between Attempts**
```
09:00 AM - Failed attempt #1 (count: 1)
09:05 AM - Failed attempt #2 (count: 2)
09:25 AM - Failed attempt #3 → ✅ Counter reset to 1 (>15 min since last attempt)
```

**Scenario 3: Successful Login Resets Counter**
```
10:00 AM - Failed attempt #1 (count: 1)
10:05 AM - Failed attempt #2 (count: 2)
10:10 AM - Successful login → Counter reset to 0, all tracking cleared
```

### Account Status Validation

#### Status Enum Values
- **pending**: Email not verified, cannot login
- **active**: Email verified, can login
- **inactive**: Account deactivated, cannot login

#### Validation Flow
1. Check account_locked_until (if set and > now, reject with "Account locked" error)
2. Check status (if not 'active', reject with "Account not active" error)
3. Check failed_login_attempts (if >= 5 within 15 min window, reject with "Too many attempts" error)
4. Proceed with password verification

---

## Security Considerations

1. **Password Security**:
   - Argon2 hashing algorithm (equivalent to 10 bcrypt salt rounds)
   - Password never logged or exposed in responses

2. **Generic Error Messages**:
   - "Invalid email or password" for both non-existent users and wrong passwords
   - Prevents email enumeration attacks

3. **Account Locking**:
   - Protects against brute force attacks
   - 5 attempts threshold with 30-minute lockout
   - Auto-reset after 15 minutes of no attempts (user-friendly)

4. **Token Security**:
   - RS256 asymmetric encryption (more secure than HS256)
   - Short-lived access tokens (15 minutes)
   - Long-lived refresh tokens (7 days) stored in Redis
   - Session-based token invalidation (logout deletes from Redis)

5. **Rate Limiting**:
   - Built-in failed attempt tracking (max 5 per 15 minutes)
   - Consider adding IP-based rate limiting at API gateway level

6. **Device Tracking**:
   - Optional device_info for security auditing
   - Enables suspicious login detection
   - Can trigger email notifications for new device logins

7. **Session Management**:
   - Each login creates unique session ID
   - Redis stores refresh token with TTL
   - Max 5 concurrent sessions per user (enforced in refresh_token logic - TODO)

---

## Testing

### Sample cURL Request

```bash
curl -X POST http://localhost:8080/v1/login_by_email \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "SecurePass123!",
    "device_info": {
      "user_agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)",
      "ip_address": "192.168.1.100"
    }
  }'
```

### Expected Workflow

#### Happy Path: Successful Login
1. User submits email and password
2. API finds user by email
3. Domain layer validates account not locked and status is active
4. Password verified using Argon2
5. Domain layer handles successful login (resets failed attempts, updates last_login_at)
6. Session ID generated and stored in Redis with 7-day TTL
7. JWT access token (15 min) and refresh token (7 days) generated
8. UserLoggedIn event published to Kafka
9. Response includes tokens and user info (id, email, full_name, role)

#### Error Path: Wrong Password
1. User submits wrong password
2. API finds user and validates account status (passes)
3. Password verification fails
4. Domain layer handles failed login (increments counter, updates timestamp)
5. User record updated in database
6. Returns 401 Unauthorized: "Invalid email or password"

#### Error Path: Account Locked
1. User attempts 6th login within 15 minutes (after 5 failures)
2. API finds user
3. Domain layer validates account → AccountMustNotBeLocked fails
4. Returns 423 Locked: "Account is temporarily locked... try again in X minutes"

#### Error Path: Account Not Active
1. User hasn't verified email (status='pending')
2. API finds user
3. Domain layer validates account → AccountMustBeActive fails
4. Returns 401 Unauthorized: "Account is not active. Please verify your email..."

---

## Integration Points

### 1. Redis
- **Stores**: Refresh tokens with session IDs
- **Key Pattern**: `refresh_token:session:{session_id}`
- **TTL**: 7 days (604800 seconds)
- **Purpose**: Session management and token invalidation on logout

### 2. Kafka
- **Topic**: `user_logged_in`
- **Consumers**:
  - Security Service (fraud detection, anomaly detection)
  - Analytics Service (DAU tracking, login patterns)
  - Notification Service (login alerts, new device notifications)
  - Audit Service (compliance logging)

### 3. Frontend Application
- **Login Form**: Email, password, (optionally collect user_agent/IP on client side)
- **Token Storage**: Store access_token in memory, refresh_token in httpOnly cookie
- **Token Refresh**: Use refresh_token to get new access_token before 15-min expiry
- **Error Handling**: Display appropriate messages for locked accounts, inactive accounts

### 4. API Gateway
- **Rate Limiting**: Consider IP-based rate limiting (e.g., 10 requests/minute per IP)
- **DDoS Protection**: Implement protection against distributed login attacks
- **Logging**: Log all login attempts with IP, user_agent for security analysis

---

## Monitoring and Observability

### Key Metrics to Track
1. **Login Success Rate**: % of successful vs failed login attempts
2. **Account Lockout Rate**: Number of accounts locked per hour/day
3. **Failed Login Distribution**: Track by user, IP, time of day
4. **Average Login Time**: Track API response time for login endpoint
5. **Session Duration**: Average time between login and logout/token expiry
6. **Device Distribution**: Track login devices/browsers for UX insights

### Log Messages
```
INFO - Login by email with request for: user@example.com
INFO - Success login for user: user@example.com
INFO - UserLoggedIn event published for user_id: 12345
WARN - Failed login attempt for user: user@example.com (attempt 3/5)
WARN - Account locked for user_id: 12345 (too many failed attempts)
ERROR - Failed to login user 'user@example.com': Invalid password
ERROR - Failed to publish UserLoggedIn event: Kafka timeout
```

### Alerts to Configure
- **High Failed Login Rate**: Alert if global failed login rate > 30% over 5 minutes
- **Account Lockout Spike**: Alert if >10 accounts locked within 1 hour (possible attack)
- **Kafka Event Failures**: Alert if UserLoggedIn events fail to publish
- **Redis Connection Issues**: Alert if refresh token storage fails

---

## Troubleshooting

### Common Issues

**Issue**: "Invalid email or password"
- **Cause**: Wrong password OR email doesn't exist OR account is pending/inactive
- **Solution**:
  - Verify email is correct
  - Check if email is verified (status must be 'active')
  - Reset password if forgotten

**Issue**: "Account is temporarily locked..."
- **Cause**: 5 failed login attempts within 15 minutes
- **Solution**: Wait for lock to expire (30 minutes from 5th attempt) OR contact support to manually unlock

**Issue**: "Account is not active. Please verify your email..."
- **Cause**: User status is 'pending' (email not verified) or 'inactive'
- **Solution**:
  - If pending: Use resend verification email endpoint
  - If inactive: Contact support for account reactivation

**Issue**: "Failed to publish UserLoggedIn event"
- **Cause**: Kafka connection issues
- **Impact**: Login succeeds but event consumers don't get notified
- **Solution**: Check Kafka cluster health, verify topic exists, check network connectivity

**Issue**: Refresh token not working
- **Cause**: Token expired (>7 days) OR session deleted from Redis (logout/manual deletion)
- **Solution**: User must login again to get new tokens

---

## Future Enhancements
- Implement refresh token rotation (one-time use tokens)
- Add IP-based rate limiting
- Implement CAPTCHA after 3 failed attempts
- Add two-factor authentication (2FA/TOTP)
- Implement "Remember me" functionality with extended refresh token
- Add social login (Google, Facebook, GitHub OAuth)
- Implement passwordless login (magic links)
- Add login history endpoint for users to view their recent logins
- Implement suspicious login detection (new device, new location)
- Add account lockout notification emails
