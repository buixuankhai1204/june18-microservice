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
