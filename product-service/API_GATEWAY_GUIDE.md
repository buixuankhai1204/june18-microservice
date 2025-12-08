# API Gateway Implementation Guide

## Overview

This API Gateway provides a **unified entry point** for all microservices in your ERP system. It handles request routing, authentication propagation, service discovery, and health monitoring.

### Key Features

- ✅ **Centralized Authentication** - JWT validation at gateway level
- ✅ **Request Proxying** - Forward requests to downstream services
- ✅ **Service Registry** - Dynamic service registration and discovery
- ✅ **Health Monitoring** - Check status of all downstream services
- ✅ **User Context Propagation** - Forward user information via headers
- ✅ **Timeout Management** - Configurable timeouts per service
- ✅ **Header Forwarding** - Smart header handling (removes hop-by-hop headers)

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         Client                              │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        │ HTTP Request with JWT
                        ▼
┌─────────────────────────────────────────────────────────────┐
│                   API Gateway (Port 3001)                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  1. JWT Authentication Middleware                    │  │
│  │  2. Extract User Claims                              │  │
│  │  3. Service Discovery (Registry Lookup)              │  │
│  │  4. Add User Context Headers (X-User-Id, X-Session)  │  │
│  │  5. Proxy Request to Downstream Service              │  │
│  └──────────────────────────────────────────────────────┘  │
└───────┬──────────────┬──────────────┬──────────────┬────────┘
        │              │              │              │
        ▼              ▼              ▼              ▼
┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌──────────────┐
│  Product    │ │   Order     │ │  Inventory  │ │ Notification │
│  Service    │ │   Service   │ │   Service   │ │   Service    │
│ (Port 3002) │ │ (Port 3003) │ │ (Port 3004) │ │ (Port 3005)  │
└─────────────┘ └─────────────┘ └─────────────┘ └──────────────┘
```

---

## Gateway Endpoints

### 1. Gateway Health Check

Check gateway status and all downstream services.

**Endpoint:** `GET /gateway/health`

**Authentication:** None (Public)

**Response:**
```json
{
  "message": "Gateway health check",
  "data": {
    "status": "healthy",
    "services": [
      {
        "name": "product-service",
        "base_url": "http://localhost:3002",
        "healthy": true
      },
      {
        "name": "order-service",
        "base_url": "http://localhost:3003",
        "healthy": true
      }
    ]
  },
  "total": 1
}
```

**Status Values:**
- `healthy` - All services are responding
- `degraded` - Some services are down

---

### 2. List Registered Services

Get all services registered in the gateway.

**Endpoint:** `GET /gateway/services`

**Authentication:** Required (JWT)

**Response:**
```json
{
  "message": "Services retrieved successfully",
  "data": [
    {
      "name": "product-service",
      "base_url": "http://localhost:3002",
      "require_auth": true
    },
    {
      "name": "notification-service",
      "base_url": "http://localhost:3005",
      "require_auth": false
    }
  ],
  "total": 1
}
```

---

### 3. Proxy Routes

The gateway exposes proxy routes for each registered service:

| Service | Gateway Route | Downstream URL |
|---------|--------------|----------------|
| Product Service | `/gateway/product-service/*` | `http://localhost:3002/*` |
| Order Service | `/gateway/order-service/*` | `http://localhost:3003/*` |
| Inventory Service | `/gateway/inventory-service/*` | `http://localhost:3004/*` |
| Notification Service | `/gateway/notification-service/*` | `http://localhost:3005/*` |

---

## How Request Proxying Works

### Example: Get Product by ID

**Client Request:**
```bash
GET /gateway/product-service/api/v1/products/123
Host: localhost:3001
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Gateway Processing:**
1. Validates JWT token
2. Extracts user claims (user_id, session_id)
3. Looks up "product-service" in registry
4. Checks if service requires authentication (yes)
5. Adds user context headers:
   - `X-User-Id: 42`
   - `X-Session-Id: 550e8400-e29b-41d4-a716-446655440000`
6. Removes hop-by-hop headers (Connection, Keep-Alive, etc.)
7. Forwards request to: `http://localhost:3002/api/v1/products/123`

**Forwarded Request to Product Service:**
```bash
GET /api/v1/products/123
Host: localhost:3002
X-User-Id: 42
X-Session-Id: 550e8400-e29b-41d4-a716-446655440000
Content-Type: application/json
```

**Response Flow:**
```
Product Service → Gateway → Client
```

---

## Registered Services (Default)

### 1. Product Service

```rust
ServiceConfig {
    name: "product-service",
    base_url: "http://localhost:3002",
    health_check_path: Some("/health"),
    timeout_secs: 30,
    require_auth: true,
}
```

**Example Routes:**
- `GET /gateway/product-service/api/v1/products`
- `GET /gateway/product-service/api/v1/products/:id`
- `POST /gateway/product-service/api/v1/products`
- `PUT /gateway/product-service/api/v1/products/:id`
- `DELETE /gateway/product-service/api/v1/products/:id`

### 2. Order Service

```rust
ServiceConfig {
    name: "order-service",
    base_url: "http://localhost:3003",
    health_check_path: Some("/health"),
    timeout_secs: 30,
    require_auth: true,
}
```

**Example Routes:**
- `GET /gateway/order-service/api/v1/orders`
- `POST /gateway/order-service/api/v1/orders`
- `GET /gateway/order-service/api/v1/orders/:id/status`

### 3. Inventory Service

```rust
ServiceConfig {
    name: "inventory-service",
    base_url: "http://localhost:3004",
    health_check_path: Some("/health"),
    timeout_secs: 30,
    require_auth: true,
}
```

**Example Routes:**
- `GET /gateway/inventory-service/api/v1/stock`
- `POST /gateway/inventory-service/api/v1/stock/reserve`

### 4. Notification Service

```rust
ServiceConfig {
    name: "notification-service",
    base_url: "http://localhost:3005",
    health_check_path: Some("/health"),
    timeout_secs: 30,
    require_auth: false,  // Public service
}
```

**Example Routes:**
- `POST /gateway/notification-service/api/v1/send`
- `GET /gateway/notification-service/api/v1/templates`

---

## Adding a New Service to Gateway

### Step 1: Register Service in Registry

Edit: `src/infrastructure/gateway/service_registry.rs`

```rust
// In ServiceRegistry::with_defaults()
registry.register(ServiceConfig {
    name: "payment-service".to_string(),
    base_url: "http://localhost:3006".to_string(),
    health_check_path: Some("/health".to_string()),
    timeout_secs: 30,
    require_auth: true,
}).await;
```

### Step 2: Create Proxy Handler

Edit: `src/infrastructure/gateway/routes.rs`

```rust
/// Proxy handler for payment service
pub async fn proxy_to_payment_service(
    State(state): State<AppState>,
    claims: Option<UserClaims>,
    request: Request,
) -> AppResult<Response> {
    proxy_to_service("payment-service", state, claims, request).await
}
```

### Step 3: Register Route

Edit: `src/api/mod.rs`

```rust
use crate::infrastructure::gateway::routes::{
    // ... existing imports
    proxy_to_payment_service,  // Add this
};

// In build_routes()
let gateway_routes = OpenApiRouter::new()
    .routes(routes!(gateway_health_check))
    .routes(routes!(list_services))
    .route("/gateway/product-service/*path", any(proxy_to_product_service))
    .route("/gateway/order-service/*path", any(proxy_to_order_service))
    .route("/gateway/inventory-service/*path", any(proxy_to_inventory_service))
    .route("/gateway/notification-service/*path", any(proxy_to_notification_service))
    .route("/gateway/payment-service/*path", any(proxy_to_payment_service));  // Add this
```

### Step 4: Restart Gateway

```bash
cargo run
```

---

## Authentication Flow

### Protected Service (require_auth: true)

```
1. Client sends request with JWT token
   ↓
2. Gateway extracts UserClaims from JWT
   ↓
3. If invalid/missing → Return 401 Unauthorized
   ↓
4. If valid → Add user context headers and proxy request
   ↓
5. Downstream service receives X-User-Id and X-Session-Id headers
```

### Public Service (require_auth: false)

```
1. Client sends request (no JWT required)
   ↓
2. Gateway checks service config (require_auth: false)
   ↓
3. Proxy request without user context headers
```

---

## User Context Headers

The gateway automatically adds these headers for authenticated requests:

| Header | Description | Example |
|--------|-------------|---------|
| `X-User-Id` | User ID from JWT claims | `42` |
| `X-Session-Id` | Session UUID from JWT | `550e8400-e29b-41d4-a716-446655440000` |

**Downstream services can use these headers:**

```rust
// In downstream service
use axum::http::HeaderMap;

async fn get_current_user(headers: HeaderMap) -> AppResult<i64> {
    let user_id = headers
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok())
        .ok_or(AppError::UnauthorizedError("Missing user ID".to_string()))?;

    Ok(user_id)
}
```

---

## Testing the Gateway

### 1. Check Gateway Health

```bash
curl http://localhost:3001/gateway/health
```

### 2. List Services (Requires JWT)

```bash
# First, login to get JWT token
curl -X POST http://localhost:3001/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "your_username",
    "password": "your_password"
  }'

# Use the access_token from response
export TOKEN="eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..."

# List services
curl http://localhost:3001/gateway/services \
  -H "Authorization: Bearer $TOKEN"
```

### 3. Proxy to Product Service

```bash
# Get all products
curl http://localhost:3001/gateway/product-service/api/v1/products \
  -H "Authorization: Bearer $TOKEN"

# Get product by ID
curl http://localhost:3001/gateway/product-service/api/v1/products/123 \
  -H "Authorization: Bearer $TOKEN"

# Create product
curl -X POST http://localhost:3001/gateway/product-service/api/v1/products \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "New Product",
    "price": 99.99,
    "description": "Product description"
  }'
```

### 4. Proxy to Public Service (No Auth)

```bash
# Send notification (public endpoint)
curl -X POST http://localhost:3001/gateway/notification-service/api/v1/send \
  -H "Content-Type: application/json" \
  -d '{
    "to": "user@example.com",
    "subject": "Test",
    "body": "Hello!"
  }'
```

---

## Error Handling

### Gateway Errors

| Error | HTTP Status | Cause | Solution |
|-------|-------------|-------|----------|
| Service not found | 404 | Service not registered | Check service name in registry |
| Authentication required | 401 | Missing/invalid JWT for protected service | Provide valid JWT token |
| Service unavailable | 400 | Downstream service is down | Check downstream service health |
| Request timeout | 400 | Request exceeded timeout | Increase timeout or optimize service |
| Proxy error | 400 | Network/connection issue | Check network connectivity |

### Example Error Response

```json
{
  "error": "Service 'product-service' not found"
}
```

---

## Configuration

### Service Configuration Options

```rust
pub struct ServiceConfig {
    pub name: String,              // Unique service identifier
    pub base_url: String,          // Downstream service URL
    pub health_check_path: Option<String>,  // Health endpoint path
    pub timeout_secs: u64,         // Request timeout in seconds
    pub require_auth: bool,        // Whether JWT is required
}
```

### Environment Variables

You can configure service URLs via environment variables:

```bash
# .env
PRODUCT_SERVICE_URL=http://product-service:3002
ORDER_SERVICE_URL=http://order-service:3003
INVENTORY_SERVICE_URL=http://inventory-service:3004
NOTIFICATION_SERVICE_URL=http://notification-service:3005
```

Then update `service_registry.rs`:

```rust
use std::env;

registry.register(ServiceConfig {
    name: "product-service".to_string(),
    base_url: env::var("PRODUCT_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:3002".to_string()),
    // ... rest of config
}).await;
```

---

## Performance Considerations

### Connection Pooling

The gateway uses `reqwest::Client` with connection pooling enabled by default:

```rust
let client = Client::builder()
    .timeout(Duration::from_secs(30))
    .build()
    .expect("Failed to create HTTP client");
```

### Timeout Configuration

Each service can have its own timeout:

```rust
ServiceConfig {
    timeout_secs: 30,  // 30 seconds for this service
}
```

### Header Filtering

The gateway removes hop-by-hop headers to prevent connection issues:
- `Connection`
- `Keep-Alive`
- `Proxy-Authenticate`
- `Proxy-Authorization`
- `TE`
- `Trailers`
- `Transfer-Encoding`
- `Upgrade`

---

## Security Best Practices

### 1. Always Use HTTPS in Production

```rust
ServiceConfig {
    base_url: "https://product-service.example.com".to_string(),
}
```

### 2. Validate JWT at Gateway Level

The gateway validates JWT tokens before proxying:
- Signature verification using RSA keys
- Expiration check
- Claims extraction

### 3. Rate Limiting (Future Enhancement)

Consider adding rate limiting per user/service:

```rust
// Future implementation
use tower::ServiceBuilder;
use tower_http::limit::RateLimitLayer;

ServiceBuilder::new()
    .layer(RateLimitLayer::new(100, Duration::from_secs(60)))
```

### 4. Service-to-Service Authentication

Downstream services should validate `X-User-Id` and `X-Session-Id`:

```rust
// In downstream service
async fn verify_gateway_request(headers: HeaderMap, redis: &RedisClient) -> AppResult<()> {
    let user_id = headers.get("X-User-Id")?;
    let session_id = headers.get("X-Session-Id")?;

    // Verify session in Redis
    redis.verify_session(user_id, session_id).await?;
    Ok(())
}
```

---

## Monitoring & Observability

### Health Check Endpoint

Use `/gateway/health` for monitoring:

```bash
# Kubernetes liveness probe
livenessProbe:
  httpGet:
    path: /gateway/health
    port: 3001
  initialDelaySeconds: 10
  periodSeconds: 30
```

### Logging

The gateway logs all proxy requests:

```
INFO Proxying GET request to: http://localhost:3002/api/v1/products (service: product-service)
```

### Metrics (Future Enhancement)

Consider adding Prometheus metrics:
- Request count per service
- Request duration
- Error rate
- Service health status

---

## Advanced Features (Future)

### 1. Circuit Breaker

Prevent cascading failures when a service is down:

```rust
// Future implementation
use tower::ServiceBuilder;
use tower::CircuitBreakerLayer;

ServiceBuilder::new()
    .layer(CircuitBreakerLayer::new(10, Duration::from_secs(30)))
```

### 2. Request/Response Transformation

Modify requests/responses before forwarding:

```rust
// Add custom headers
headers.insert("X-Gateway-Version", "1.0");

// Transform response body
let transformed_body = transform_json(response_body)?;
```

### 3. Load Balancing

Support multiple instances of a service:

```rust
ServiceConfig {
    name: "product-service",
    instances: vec![
        "http://product-1:3002",
        "http://product-2:3002",
        "http://product-3:3002",
    ],
    load_balancer: LoadBalancerStrategy::RoundRobin,
}
```

### 4. API Key Management

Support multiple authentication methods:

```rust
enum AuthMethod {
    JWT,
    ApiKey,
    OAuth2,
}
```

---

## Troubleshooting

### Gateway won't start

```bash
# Check if port 3001 is available
lsof -i :3001

# Check dependencies
cargo check
```

### Service health check fails

```bash
# Verify downstream service is running
curl http://localhost:3002/health

# Check network connectivity
ping localhost
```

### Requests timing out

```bash
# Increase timeout in service config
timeout_secs: 60  // Increase from 30 to 60

# Check downstream service performance
curl -w "@curl-format.txt" http://localhost:3002/api/v1/products
```

### Authentication errors

```bash
# Verify JWT token is valid
jwt decode $TOKEN

# Check token expiration
# Regenerate token if expired
```

---

## File Structure

```
src/
├── infrastructure/
│   └── gateway/
│       ├── mod.rs                    # Module exports
│       ├── service_registry.rs       # Service discovery & registration
│       ├── proxy.rs                  # HTTP proxy logic
│       └── routes.rs                 # Gateway route handlers
├── core/
│   └── app_state.rs                  # Add GatewayState
└── api/
    └── mod.rs                        # Register gateway routes
```

---

## Complete Example: Adding Payment Service

### 1. Start Payment Service (Separate Project)

```rust
// payment-service/src/main.rs
use axum::{routing::get, Router, Json};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/payments", get(list_payments));

    axum::Server::bind(&"0.0.0.0:3006".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}

async fn list_payments() -> Json<Vec<Payment>> {
    Json(vec![])
}
```

### 2. Register in Gateway

```rust
// src/infrastructure/gateway/service_registry.rs
registry.register(ServiceConfig {
    name: "payment-service".to_string(),
    base_url: "http://localhost:3006".to_string(),
    health_check_path: Some("/health".to_string()),
    timeout_secs: 30,
    require_auth: true,
}).await;
```

### 3. Test

```bash
# Check health
curl http://localhost:3001/gateway/health

# Should show payment-service in the list

# Access payment service through gateway
curl http://localhost:3001/gateway/payment-service/api/v1/payments \
  -H "Authorization: Bearer $TOKEN"
```

---

**Last Updated:** 2025-12-04
