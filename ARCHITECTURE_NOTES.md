# Architecture Notes - User Registration Feature

## Business Rules Implementation Strategy

### Key Principle: Separation of Concerns

Business rules are strategically placed based on their dependencies:

#### 1. Domain Layer Business Rules
**Location**: `src/domain/user/user.rs` → `ModelEx::create_user_for_registration()`

**Rules enforced in domain layer** (no external dependencies):
- ✅ `EmailMustBeValid` - Email format validation (regex)
- ✅ `PasswordMustMeetRequirements` - Password complexity validation
- ✅ `FullNameMustBeValid` - Full name validation
- ✅ `PhoneMustBeValid` - Phone format validation
- ✅ `UserMustBeAtLeastAge` - Age validation (13+ years)

**Why in domain layer?**
- These rules are **pure business logic** with no external dependencies
- Can be tested in isolation without database or infrastructure
- Domain layer remains independent and portable
- Follows DDD principle: "Domain model is the source of truth for business rules"

#### 2. Application Layer Business Rules
**Location**: `src/application/user/user_service.rs` → `UserService::register_user()`

**Rules enforced in application layer** (require database access):
- ✅ `EmailMustBeUnique` - Requires database query to check existence
- ✅ `PhoneMustBeUnique` - Requires database query to check existence

**Why in application layer?**
- These rules **depend on database infrastructure**
- Cannot be validated without database connection
- Application service orchestrates infrastructure access
- Checked BEFORE calling domain layer to fail fast

### Flow Diagram

```
Controller (API Layer)
    ↓ receives RegisterUserCommand
Application Service
    ↓ validates EmailMustBeUnique (DB query)
    ↓ validates PhoneMustBeUnique (DB query)
    ↓ calls domain layer
Domain Layer (ModelEx::create_user_for_registration)
    ↓ validates EmailMustBeValid
    ↓ validates PasswordMustMeetRequirements
    ↓ validates FullNameMustBeValid
    ↓ validates PhoneMustBeValid
    ↓ validates UserMustBeAtLeastAge
    ↓ returns validated User model
Application Service
    ↓ hashes password
    ↓ generates verification token
    ↓ persists to database
    ↓ publishes event
    ↓ returns UserCreatedSerializer
```

## Benefits of This Approach

### 1. **Testability**
```rust
// Domain layer can be tested without database
#[test]
fn test_invalid_email_format() {
    let result = ModelEx::create_user_for_registration(
        "invalid-email".to_string(), // Invalid format
        "SecurePass123!".to_string(),
        "John Doe".to_string(),
        None,
        None,
    );

    assert!(result.is_err()); // Fails at domain layer
}
```

### 2. **Single Responsibility**
- **Domain**: Business logic and invariants
- **Application**: Orchestration and database-dependent rules
- **Infrastructure**: Database operations
- **Presentation**: HTTP/API concerns

### 3. **Fail Fast Principle**
Application layer checks database-dependent rules FIRST (email/phone uniqueness) before invoking expensive domain operations.

### 4. **Domain Integrity**
Domain layer cannot create an invalid user model - all invariants are enforced at creation time.

## Common Patterns

### Pattern 1: Business Rule Classes
Each rule is a separate class in `src/domain/user/rules/`:
```rust
pub struct EmailMustBeValid {
    pub email: String,
}

impl BusinessRuleInterface for EmailMustBeValid {
    fn check_broken(&self) -> AppResult<()> {
        // Validation logic
    }
}
```

### Pattern 2: Rule Invocation in Domain
```rust
// In ModelEx::create_user_for_registration()
EmailMustBeValid { email: email.clone() }.check_broken()?;
PasswordMustMeetRequirements { password: password.clone() }.check_broken()?;
```

### Pattern 3: Database-Dependent Rules in Application
```rust
// In UserService::register_user()
let email_is_unique = !user::user::Entity::email_exists(conn, &command.email).await?;
EmailMustBeUnique { is_unique: email_is_unique }.check_broken()?;
```

## File Structure Summary

```
src/
├── domain/
│   └── user/
│       ├── user.rs                          # ModelEx::create_user_for_registration()
│       ├── rules/
│       │   ├── email_must_be_valid.rs      # Domain rule
│       │   ├── email_must_be_unique.rs     # Application rule (used in service)
│       │   ├── password_must_meet_requirements.rs  # Domain rule
│       │   ├── full_name_must_be_valid.rs  # Domain rule
│       │   ├── phone_must_be_valid.rs      # Domain rule
│       │   ├── phone_must_be_unique.rs     # Application rule (used in service)
│       │   └── user_must_be_at_least_age.rs # Domain rule
│       ├── events/
│       │   └── user_registered.rs          # Domain event
│       └── verification.rs                  # Domain helper
│
├── application/
│   └── user/
│       ├── user_command.rs                  # RegisterUserCommand
│       ├── user_service.rs                  # Orchestration + DB-dependent rules
│       └── user_service_interface.rs        # Contract
│
├── infrastructure/
│   └── model/
│       └── user_repository.rs               # email_exists(), phone_exists()
│
├── presentation/
│   └── user/
│       └── user.rs                          # UserCreatedSerializer
│
└── api/
    └── domain/
        └── user/
            └── user.rs                      # controller_register_user
```

---

## FR-US-002: Email Verification Implementation

### Overview
Email verification allows users to verify their email address and activate their account. Includes resend functionality with rate limiting.

### Business Rules Implementation

#### 1. Domain Layer Business Rules
**Location**: `src/domain/user/user.rs` → `ModelEx::verify_email()` and `ModelEx::prepare_resend_verification()`

**Rules enforced in domain layer** (no external dependencies):
- ✅ `VerificationTokenMustNotBeExpired` - Validates token expiry (24h limit)
- ✅ `UserMustNotBeAlreadyVerified` - Ensures user isn't already verified
- ✅ `VerificationResendLimitMustNotBeExceeded` - Enforces max 3 resends per hour with auto-reset logic

**Why in domain layer?**
- Token expiration check is **pure business logic** (timestamp comparison)
- Already-verified check is **domain invariant** (user state validation)
- Rate limiting logic with counter reset is **core business rule** (no external service needed)
- Can be tested in isolation without database

#### 2. Application Layer Business Rules
**Location**: `src/application/user/user_service.rs` → `UserService::verify_email()` and `UserService::resend_verification_email()`

**Rules enforced in application layer** (require database access):
- ✅ `VerificationTokenMustExist` - Requires database query to find user by token
- ✅ User lookup by email - Database operation to find user for resend

**Why in application layer?**
- Finding user by token **requires database query**
- Finding user by email **requires database query**
- Application service orchestrates database access before domain validation

### Flow Diagram - Email Verification

```
Controller (API Layer)
    ↓ receives VerifyEmailCommand
Application Service
    ↓ finds user by verification_token (DB query)
    ↓ validates VerificationTokenMustExist
    ↓ calls domain layer
Domain Layer (ModelEx::verify_email)
    ↓ validates UserMustNotBeAlreadyVerified
    ↓ validates VerificationTokenMustNotBeExpired
    ↓ updates status: PENDING → ACTIVE
    ↓ sets email_verified_at timestamp
    ↓ clears verification_token and expiry
    ↓ returns verified User model
Application Service
    ↓ persists updated user to database
    ↓ publishes UserActivated event
    ↓ returns success
```

### Flow Diagram - Resend Verification Email

```
Controller (API Layer)
    ↓ receives ResendVerificationEmailCommand
Application Service
    ↓ finds user by email (DB query)
    ↓ generates new verification token
    ↓ calls domain layer
Domain Layer (ModelEx::prepare_resend_verification)
    ↓ checks if >1 hour passed since last resend
    ↓ resets counter to 0 if >1 hour (auto-reset logic)
    ↓ validates UserMustNotBeAlreadyVerified
    ↓ validates VerificationResendLimitMustNotBeExceeded
    ↓ updates verification_token and expiry
    ↓ increments verification_resend_count
    ↓ updates last_verification_resend_at
    ↓ returns updated User model
Application Service
    ↓ persists updated user to database
    ↓ publishes UserRegistered event (reuses notification flow)
    ↓ returns success
```

### Key Pattern: Rate Limiting with Auto-Reset

**Implementation in Domain Layer** (`src/domain/user/user.rs`):
```rust
pub fn prepare_resend_verification(
    mut self,
    new_token: String,
    new_expiry: NaiveDateTime
) -> AppResult<Self> {
    let now = Utc::now().naive_utc();

    // Auto-reset logic: Reset counter if >1 hour passed
    if let Some(last_resend) = self.last_verification_resend_at {
        let one_hour_ago = now - Duration::hours(1);
        if last_resend <= one_hour_ago {
            self.verification_resend_count = 0; // Counter reset
        }
    }

    // Now enforce rate limit with potentially reset counter
    VerificationResendLimitMustNotBeExceeded {
        resend_count: self.verification_resend_count,
        last_resend_at: self.last_verification_resend_at,
        max_resends_per_hour: 3,
    }.check_broken()?;

    // Update model with new token and increment counter
    self.verification_token = Some(new_token);
    self.verification_token_expiry = Some(new_expiry);
    self.verification_resend_count += 1;
    self.last_verification_resend_at = Some(now);

    Ok(self)
}
```

**Why this pattern?**
- **Auto-reset is business logic**, not infrastructure concern
- **Domain model controls its own state transitions**
- **No external timer/scheduler needed** - reset happens on next request
- **User-friendly** - users don't need to manually wait for exact reset time

### Key Pattern: Domain Events for Integration

**UserActivated Event**:
```rust
// Published when email verification succeeds
let event = UserActivatedEvent::new(
    user_id,
    user_email,
    verified_at,
);

// Kafka topic: "user_activated"
// Consumed by: Notification Service (welcome email), Analytics, etc.
```

**Reusing UserRegistered Event for Resend**:
```rust
// Reuses same event type for consistency
let event = UserRegisteredEvent::new(
    updated_user.id,
    updated_user.email.clone(),
    full_name,
    new_token, // New token for resend
    chrono::Utc::now().naive_utc(),
);

// Kafka topic: "user_registered"
// Notification service handles both initial and resend the same way
```

**Why reuse UserRegistered for resend?**
- **Single notification handler** - no need to duplicate email sending logic
- **Same email template** - verification emails are identical
- **Simpler architecture** - fewer event types to maintain

### Database Schema Evolution

**Migration File**: `user_migration/src/m20251209_000000_add_email_verification_resend_tracking.rs`

**Fields Added for FR-US-002** (ALTER TABLE migration):
```sql
ALTER TABLE users
ADD COLUMN verification_resend_count INTEGER DEFAULT 0 NOT NULL;

ALTER TABLE users
ADD COLUMN last_verification_resend_at TIMESTAMP NULL;
```

**Purpose**:
- `verification_resend_count`: Tracks resends within current window
- `last_verification_resend_at`: Enables auto-reset logic (>1 hour check)

**Migration Strategy**:
- ✅ Used ALTER TABLE for existing databases (not CREATE TABLE)
- ✅ Includes rollback support via `down()` method
- ✅ Run with: `cd user_migration && cargo run -- up`

### File Structure for FR-US-002

```
src/
├── domain/
│   └── user/
│       ├── user.rs
│       │   ├── verify_email()                    # Domain method
│       │   └── prepare_resend_verification()     # Domain method with auto-reset
│       ├── rules/
│       │   ├── verification_token_must_exist.rs           # Application rule
│       │   ├── verification_token_must_not_be_expired.rs  # Domain rule
│       │   ├── user_must_not_be_already_verified.rs       # Domain rule
│       │   └── verification_resend_limit_must_not_be_exceeded.rs  # Domain rule
│       ├── events/
│       │   └── user_activated.rs                # New domain event
│       └── verification.rs
│           ├── generate_verification_token()    # Token generation helper
│           └── is_token_expired()              # Expiry check helper
│
├── application/
│   └── user/
│       ├── user_command.rs
│       │   ├── VerifyEmailCommand              # New command
│       │   └── ResendVerificationEmailCommand  # New command
│       └── user_service.rs
│           ├── verify_email()                  # Service method
│           └── resend_verification_email()     # Service method
│
├── infrastructure/
│   └── model/
│       └── user_repository.rs
│           └── find_user_by_verification_token()  # New repository method
│
└── api/
    └── domain/
        └── user/
            └── user.rs
                ├── controller_verify_email()              # New endpoint
                └── controller_resend_verification_email() # New endpoint
```

---

## FR-US-003: User Login Implementation

### Overview
User login authenticates users and issues JWT tokens. Includes failed login tracking, account locking, and device information logging for security.

### Business Rules Implementation

#### 1. Domain Layer Business Rules
**Location**: `src/domain/user/user.rs` → `ModelEx::validate_login_attempt()`, `ModelEx::handle_failed_login()`, `ModelEx::handle_successful_login()`

**Rules enforced in domain layer** (no external dependencies):
- ✅ `AccountMustNotBeLocked` - Validates account_locked_until < now
- ✅ `AccountMustBeActive` - Validates user status == 'active'
- ✅ `FailedLoginLimitMustNotBeExceeded` - Validates failed attempts < 5 within 15-minute window with auto-reset

**Why in domain layer?**
- Account lock check is **pure business logic** (timestamp comparison)
- Status validation is **domain invariant** (user state validation)
- Failed login tracking with auto-reset is **core business rule** (no external service needed)
- Can be tested in isolation without database

#### 2. Application Layer Operations
**Location**: `src/application/authen/authen_service.rs` → `AuthenService::login_by_email()`

**Operations requiring infrastructure** (not business rules):
- ✅ Find user by email - Database query
- ✅ Password verification - Argon2 hashing (external library)
- ✅ Session storage - Redis operations
- ✅ JWT token generation - Cryptography library
- ✅ Event publishing - Kafka producer

**Why in application layer?**
- These are **infrastructure operations**, not business rules
- Require external dependencies (database, Redis, Kafka)
- Application service orchestrates infrastructure and domain

### Flow Diagram - User Login

```
Controller (API Layer)
    ↓ receives LoginByEmailCommand
Application Service
    ↓ finds user by email (DB query)
    ↓ if not found → return "Invalid email or password"
    ↓ calls domain layer validation
Domain Layer (ModelEx::validate_login_attempt)
    ↓ validates AccountMustNotBeLocked
    ↓ validates AccountMustBeActive
    ↓ validates FailedLoginLimitMustNotBeExceeded
    ↓ returns validation result
Application Service
    ↓ verifies password using Argon2
    ↓ if password invalid:
        ↓ calls domain layer
        Domain Layer (ModelEx::handle_failed_login)
            ↓ checks if >15 min since last failure
            ↓ resets counter if >15 min (auto-reset)
            ↓ increments failed_login_attempts
            ↓ updates last_failed_login_at
            ↓ locks account if attempts >= 5 (sets account_locked_until)
            ↓ returns updated User model
        ↓ persists to database
        ↓ returns 401 Unauthorized
    ↓ if password valid:
        ↓ calls domain layer
        Domain Layer (ModelEx::handle_successful_login)
            ↓ resets failed_login_attempts to 0
            ↓ clears last_failed_login_at
            ↓ clears account_locked_until
            ↓ updates last_login_at
            ↓ returns updated User model
        ↓ persists to database
        ↓ generates session ID (UUID)
        ↓ stores refresh token in Redis (7 days TTL)
        ↓ generates JWT access token (15 min) and refresh token (7 days)
        ↓ publishes UserLoggedIn event to Kafka
        ↓ returns TokenResponse with user info
```

### Key Pattern: Auto-Reset Failed Login Counter

**Implementation in Domain Layer** (`src/domain/user/user.rs`):
```rust
pub fn handle_failed_login(mut self) -> Self {
    let now = Utc::now().naive_utc();

    // Auto-reset logic: Reset counter if >15 minutes passed
    if let Some(last_failed) = self.last_failed_login_at {
        let fifteen_minutes_ago = now - Duration::minutes(15);
        if last_failed <= fifteen_minutes_ago {
            self.failed_login_attempts = 0; // Counter reset
        }
    }

    // Increment failed login counter
    self.failed_login_attempts += 1;
    self.last_failed_login_at = Some(now);

    // Lock account for 30 minutes if 5 or more failed attempts
    if self.failed_login_attempts >= 5 {
        self.account_locked_until = Some(now + Duration::minutes(30));
    }

    self.updated_at = Some(now);
    self
}
```

**Why this pattern?**
- **Auto-reset is business logic**, not infrastructure concern
- **Domain model controls its own state transitions**
- **No external timer/scheduler needed** - reset happens on next login attempt
- **User-friendly** - users get automatic retry window after 15 minutes of no attempts

### Key Pattern: Separation of Validation and Side Effects

**Domain Layer** - Pure validation (no side effects):
```rust
// VALIDATES but doesn't modify anything
pub fn validate_login_attempt(&self) -> AppResult<()> {
    AccountMustNotBeLocked { ... }.check_broken()?;
    AccountMustBeActive { ... }.check_broken()?;
    FailedLoginLimitMustNotBeExceeded { ... }.check_broken()?;
    Ok(())
}
```

**Domain Layer** - State transitions (pure, returns new state):
```rust
// MODIFIES state but has no external dependencies
pub fn handle_failed_login(mut self) -> Self {
    // Pure state transformation
    self.failed_login_attempts += 1;
    self.last_failed_login_at = Some(now);
    // ...
    self
}
```

**Application Layer** - Infrastructure operations (side effects):
```rust
// ORCHESTRATES infrastructure and domain
async fn login_by_email(...) -> AppResult<TokenResponse> {
    let user = find_user_by_email(conn, email).await?;  // DB query
    user.validate_login_attempt()?;                      // Domain validation
    let valid = verify_password(...).await?;              // Argon2
    let user = user.handle_failed_login();                // Domain transformation
    update_user(conn, user).await?;                       // DB persistence
    store_in_redis(...).await?;                           // Redis operation
    publish_to_kafka(...).await?;                         // Kafka operation
    Ok(response)
}
```

**Why this separation?**
- **Domain layer is pure and testable** - no mocking needed
- **Application layer handles all I/O** - database, Redis, Kafka
- **Clear responsibilities** - domain = rules, application = orchestration

### Database Schema Evolution

**Migration File**: `user_migration/src/m20251209_000001_add_login_tracking_fields.rs`

**Fields Added for FR-US-003** (ALTER TABLE migration):
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
- `failed_login_attempts`: Tracks failed attempts within 15-minute rolling window
- `last_failed_login_at`: Enables auto-reset logic (>15 min check)
- `account_locked_until`: Timestamp for account lockout expiration
- `last_login_at`: Tracks last successful login for analytics/security

**Migration Strategy**:
- ✅ Used ALTER TABLE for existing databases
- ✅ Includes rollback support via `down()` method
- ✅ Default values ensure existing users work without migration issues
- ✅ Run with: `cd user_migration && cargo run -- up`

### JWT Token Strategy

**Access Token** (Short-lived):
- **Expiry**: 15 minutes (900 seconds)
- **Purpose**: Authorize API requests
- **Storage**: Memory/localStorage (client-side)
- **Algorithm**: RS256 (RSA asymmetric encryption)

**Refresh Token** (Long-lived):
- **Expiry**: 7 days (604800 seconds)
- **Purpose**: Obtain new access tokens without re-login
- **Storage**: Redis with key `refresh_token:session:{session_id}`
- **Algorithm**: RS256 (RSA asymmetric encryption)

**Session Management**:
- Each login creates unique session_id (UUID v4)
- Both tokens contain same session_id in claims
- Logout deletes session from Redis → invalidates both tokens
- Future: Implement max 5 concurrent sessions per user

### File Structure for FR-US-003

```
src/
├── domain/
│   └── user/
│       ├── user.rs
│       │   ├── validate_login_attempt()        # Domain validation
│       │   ├── handle_failed_login()           # Domain state transition
│       │   └── handle_successful_login()       # Domain state transition
│       ├── rules/
│       │   ├── account_must_be_active.rs                    # Domain rule
│       │   ├── account_must_not_be_locked.rs                # Domain rule
│       │   └── failed_login_limit_must_not_be_exceeded.rs   # Domain rule
│       └── events/
│           └── user_logged_in.rs               # Domain event
│
├── application/
│   └── authen/
│       ├── authen_command.rs
│       │   ├── LoginByEmailCommand             # Command with device_info
│       │   └── DeviceInfo                      # Device tracking DTO
│       ├── authen_service.rs
│       │   └── login_by_email()                # Service orchestration
│       └── claim.rs
│           └── service_generate_tokens()       # JWT generation
│
├── infrastructure/
│   ├── error.rs
│   │   └── AccountLockedError                  # New error variant (423 status)
│   └── model/
│       └── user_repository.rs
│           └── find_user_by_email()            # Repository method
│
├── presentation/
│   └── authen/
│       └── authen.rs
│           ├── TokenResponse                   # Updated with user info
│           └── UserInfo                        # User details in response
│
└── api/
    └── domain/
        └── auth/
            └── auth.rs
                └── controller_login_by_email() # Updated endpoint
```

---

## Key Takeaways

1. ✅ **Business rules WITHOUT external dependencies** → Domain Layer
2. ✅ **Business rules WITH database/external dependencies** → Application Layer
3. ✅ Domain layer is **pure and testable**
4. ✅ Application layer **orchestrates** domain and infrastructure
5. ✅ Each layer has **clear responsibilities**
6. ✅ Business rules are **explicit and self-documenting**
7. ✅ **Auto-reset logic in domain** - no external schedulers needed
8. ✅ **Reuse events** where appropriate to simplify architecture
9. ✅ **Domain events enable loose coupling** between services
10. ✅ **Separate validation from state transitions** - testability and clarity
11. ✅ **Security through domain rules** - failed login tracking, account locking
12. ✅ **Infrastructure operations in application layer** - Redis, Kafka, JWT
