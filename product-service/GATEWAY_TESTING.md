# API Gateway Testing Guide

## Overview

The API Gateway has been successfully integrated into your user-service. It now acts as a reverse proxy to route requests to other microservices (NestJS or other frameworks).

## What Was Implemented

### 1. Gateway Components

- **Service Registry** (`src/infrastructure/gateway/service_registry.rs`)
  - Manages registration of downstream services
  - Pre-configured with 4 default services: product, order, inventory, notification

- **Proxy Handler** (`src/infrastructure/gateway/proxy.rs`)
  - Handles HTTP request forwarding
  - Filters hop-by-hop headers
  - Adds user context headers (X-User-Id, X-Session-Id)
  - Manages timeouts per service

- **Gateway Routes** (`src/infrastructure/gateway/routes.rs`)
  - Health check endpoint
  - Service listing endpoint
  - Proxy handlers for each registered service

### 2. Gateway Endpoints

| Endpoint | Method | Auth Required | Description |
|----------|--------|---------------|-------------|
| `/gateway/health` | GET | No | Check gateway and all services health |
| `/gateway/services` | GET | Yes (JWT) | List all registered services |
| `/gateway/product-service/*` | ANY | Yes | Proxy to product service |
| `/gateway/order-service/*` | ANY | Yes | Proxy to order service |
| `/gateway/inventory-service/*` | ANY | Yes | Proxy to inventory service |
| `/gateway/notification-service/*` | ANY | No | Proxy to notification service |

## Configuration

### Default Service Registration

The gateway pre-registers these services (see `service_registry.rs:35-75`):

```rust
// Product Service - http://localhost:3002
// Order Service - http://localhost:3003
// Inventory Service - http://localhost:3004
// Notification Service - http://localhost:3005 (public, no auth)
```

### Environment Variables (Optional)

You can override service URLs via environment variables in your `.env`:

```bash
PRODUCT_SERVICE_URL=http://localhost:3002
ORDER_SERVICE_URL=http://localhost:3003
INVENTORY_SERVICE_URL=http://localhost:3004
NOTIFICATION_SERVICE_URL=http://localhost:3005
```

## Testing the Gateway

### 1. Start the Gateway (User Service)

```bash
cargo run
# Server should start on port 3001 (or your configured port)
```

### 2. Test Health Check (No Auth Required)

```bash
curl http://localhost:3001/gateway/health | jq
```

Expected response:
```json
{
  "message": "Gateway health check",
  "data": {
    "status": "degraded",
    "services": [
      {
        "name": "product-service",
        "base_url": "http://localhost:3002",
        "healthy": false
      },
      {
        "name": "order-service",
        "base_url": "http://localhost:3003",
        "healthy": false
      },
      {
        "name": "inventory-service",
        "base_url": "http://localhost:3004",
        "healthy": false
      },
      {
        "name": "notification-service",
        "base_url": "http://localhost:3005",
        "healthy": false
      }
    ]
  },
  "total": 1
}
```

> **Note**: Services will show `healthy: false` until you start the downstream NestJS services.

### 3. Create a Simple NestJS Service for Testing

Create a test NestJS service on port 3002:

```bash
# In a new directory
npm i -g @nestjs/cli
nest new product-service
cd product-service
```

Edit `src/main.ts`:
```typescript
import { NestFactory } from '@nestjs/core';
import { AppModule } from './app.module';

async function bootstrap() {
  const app = await NestFactory.create(AppModule);
  await app.listen(3002);
  console.log('Product service running on http://localhost:3002');
}
bootstrap();
```

Edit `src/app.controller.ts`:
```typescript
import { Controller, Get, Post, Body, Param, Headers } from '@nestjs/common';
import { AppService } from './app.service';

@Controller()
export class AppController {
  constructor(private readonly appService: AppService) {}

  @Get('health')
  getHealth() {
    return { status: 'OK', service: 'product-service' };
  }

  @Get('api/v1/products')
  getProducts(@Headers() headers: Record<string, string>) {
    console.log('User ID from gateway:', headers['x-user-id']);
    console.log('Session ID from gateway:', headers['x-session-id']);

    return {
      message: 'Products retrieved',
      data: [
        { id: 1, name: 'Product 1', price: 100 },
        { id: 2, name: 'Product 2', price: 200 },
      ],
      total: 2
    };
  }

  @Get('api/v1/products/:id')
  getProduct(@Param('id') id: string, @Headers() headers: Record<string, string>) {
    console.log('User ID from gateway:', headers['x-user-id']);
    return {
      message: 'Product retrieved',
      data: { id: parseInt(id), name: `Product ${id}`, price: 100 },
      total: 1
    };
  }

  @Post('api/v1/products')
  createProduct(@Body() body: any, @Headers() headers: Record<string, string>) {
    console.log('User ID from gateway:', headers['x-user-id']);
    console.log('Product data:', body);
    return {
      message: 'Product created',
      data: { id: 3, ...body },
      total: 1
    };
  }
}
```

Start the NestJS service:
```bash
npm run start:dev
```

### 4. Test Gateway Health Again

```bash
curl http://localhost:3001/gateway/health | jq
```

Now product-service should show `healthy: true`.

### 5. Test Authentication

First, get a JWT token by logging in:

```bash
curl -X POST http://localhost:3001/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "your_email@example.com",
    "password": "your_password"
  }' | jq
```

Save the `access_token` from the response:
```bash
export TOKEN="<your_access_token_here>"
```

### 6. List Services (Requires Auth)

```bash
curl http://localhost:3001/gateway/services \
  -H "Authorization: Bearer $TOKEN" | jq
```

### 7. Proxy Requests to NestJS Product Service

**Get all products:**
```bash
curl http://localhost:3001/gateway/product-service/api/v1/products \
  -H "Authorization: Bearer $TOKEN" | jq
```

**Get product by ID:**
```bash
curl http://localhost:3001/gateway/product-service/api/v1/products/123 \
  -H "Authorization: Bearer $TOKEN" | jq
```

**Create a product:**
```bash
curl -X POST http://localhost:3001/gateway/product-service/api/v1/products \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "New Product",
    "price": 299.99,
    "description": "A great product"
  }' | jq
```

### 8. Check NestJS Logs

In your NestJS service terminal, you should see logs showing the user context headers:

```
User ID from gateway: 42
Session ID from gateway: 550e8400-e29b-41d4-a716-446655440000
```

## How It Works

### Request Flow

```
Client
  ↓ (HTTP Request with JWT)
API Gateway (User Service - Port 3001)
  ↓
1. Extract JWT token from Authorization header
2. Decode JWT to get user_id and session_id (sid)
3. Look up service in registry
4. Check if service requires authentication
5. Add headers: X-User-Id, X-Session-Id
6. Forward request to downstream service
  ↓
NestJS Service (Port 3002)
  ↓ (receives request with user context headers)
Process request using X-User-Id and X-Session-Id
  ↓ (response)
API Gateway
  ↓ (forward response)
Client
```

### User Context Propagation

The gateway automatically adds these headers to authenticated requests:

- `X-User-Id`: The user's ID from the JWT claims (e.g., "42")
- `X-Session-Id`: The session UUID from the JWT claims (e.g., "550e8400...")

Your NestJS services can read these headers to identify the current user without needing to validate the JWT themselves.

## Adding New Services

To add a new service (e.g., payment-service):

### 1. Register in Service Registry

Edit `src/infrastructure/gateway/service_registry.rs:35`:

```rust
// Add after notification-service
registry
    .register(ServiceConfig {
        name: "payment-service".to_string(),
        base_url: std::env::var("PAYMENT_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:3006".to_string()),
        health_check_path: Some("/health".to_string()),
        timeout_secs: 30,
        require_auth: true,
    })
    .await;
```

### 2. Create Proxy Handler

Edit `src/infrastructure/gateway/routes.rs:193`:

```rust
pub async fn proxy_to_payment_service(
    State(state): State<AppState>,
    request: Request,
) -> AppResult<Response<Body>> {
    let claims = extract_claims_from_request(&request);
    proxy_to_service("payment-service", state, claims, request).await
}
```

### 3. Register Route

Edit `src/api/mod.rs:41`:

```rust
.route("/gateway/payment-service/*path", any(proxy_to_payment_service))
```

### 4. Rebuild and Test

```bash
cargo build
cargo run
```

## Security Considerations

1. **JWT Validation**: The gateway validates JWTs before forwarding requests
2. **Header Filtering**: Hop-by-hop headers are removed to prevent connection issues
3. **Service-Level Auth**: Each service can be configured with `require_auth: true/false`
4. **Timeout Protection**: Each service has a configurable timeout to prevent hanging requests

## Troubleshooting

### Service shows unhealthy

Check if the downstream service is running:
```bash
curl http://localhost:3002/health
```

### 401 Unauthorized

- Ensure you're passing the JWT token: `-H "Authorization: Bearer $TOKEN"`
- Check if your token is still valid (not expired)
- Verify the service has `require_auth: true` in its config

### Request timeout

- Increase `timeout_secs` in the service configuration
- Check if the downstream service is slow or hung

### Connection refused

- Verify the service URL and port are correct
- Ensure the downstream service is actually running
- Check firewall settings

## Next Steps

1. **Add more services**: Register your other NestJS microservices
2. **Environment configuration**: Use `.env` to configure service URLs per environment
3. **Add circuit breaker**: Prevent cascading failures (future enhancement)
4. **Add rate limiting**: Protect against abuse (future enhancement)
5. **Add metrics**: Monitor gateway performance (future enhancement)

## Reference

- Full API Gateway guide: `API_GATEWAY_GUIDE.md`
- Service registry: `src/infrastructure/gateway/service_registry.rs`
- Proxy handler: `src/infrastructure/gateway/proxy.rs`
- Gateway routes: `src/infrastructure/gateway/routes.rs`
