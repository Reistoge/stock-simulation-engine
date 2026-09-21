# Service Layer Pattern

This document explains how to create new services following the pattern established in `src/service/user.rs`.

## Overview

Services encapsulate business logic, separating it from:
- HTTP handlers (`src/routes/`)
- Data access (`src/repositories/`)
- Pure utilities (`src/auth/validation.rs`)

## Structure

```
src/service/
├── mod.rs           # Public exports
├── SERVICE.md       # This file
└── user.rs          # Example service
```

## Creating a New Service

### 1. Define the Service Struct

```rust
// src/service/order.rs
use crate::repositories::order::OrderRepository;
use crate::auth::types::Order;

pub struct OrderService<R: OrderRepository> {
    repo: R,
}

impl<R: OrderRepository> OrderService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_order(self, input: CreateOrderInput) -> Result<Order, StatusCode> {
        // Business logic here
        self.repo.create(input).await
    }
}
```

### 2. Use Repository Traits (Not Concrete Types)

```rust
// Good: Generic over trait
pub struct OrderService<R: OrderRepository> { repo: R }

// Avoid: Concrete type
pub struct OrderService { repo: OrderRepositoryImpl }
```

**Why?** Enables testing with `MockOrderRepository`.

### 3. Consume Self (Not &mut self)

```rust
// Good: Takes ownership
pub async fn create_order(self, input: CreateOrderInput) -> Result<Order, StatusCode>

// Avoid: Mutable reference
pub async fn create_order(&mut self, input: CreateOrderInput) -> Result<Order, StatusCode>
```

**Why?** Matches per-request instantiation in handlers; no need to hold service across requests.

### 4. Return StatusCode Errors

```rust
use axum::http::StatusCode;

pub async fn create_order(self, input: CreateOrderInput) -> Result<Order, StatusCode> {
    if input.items.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    // ...
}
```

**Why?** Simple, matches HTTP semantics, no custom error types needed yet.

### 5. Delegate to Utilities for Cross-Cutting Concerns

```rust
use crate::auth::validation::{hash_password, create_jwt};

pub async fn register(self, input: RegisterInput) -> Result<User, StatusCode> {
    let hash = hash_password(&input.password)?;  // Pure utility
    let token = create_jwt(&input.email)?;       // Pure utility
    // ...
}
```

**Why?** Keeps services focused on business logic; utilities are testable in isolation.

## Wiring in HTTP Handlers

```rust
// src/routes/order.rs
use crate::service::order::OrderService;

#[utoipa::path(post, path = "/orders", ...)]
pub async fn create_order(
    State(app): State<AppState>,
    Json(input): Json<CreateOrderInput>,
) -> Result<Json<Order>, StatusCode> {
    let service = OrderService::new(app.order_repository);
    let order = service.create_order(input).await?;
    Ok(Json(order))
}
```

**Key points:**
- Instantiate service per-request with `app.repository`
- Extract data from HTTP (headers, body, path) in handler
- Pass clean domain types to service
- Return `Json<T>` or `StatusCode`

## Testing

### Unit Tests (Pure Utilities)
```rust
// src/auth/validation.rs
#[cfg(test)]
mod tests {
    #[test]
    fn hash_password_roundtrip() { ... }
}
```

### Integration Tests (Service)
```rust
// src/service/order.rs
#[cfg(test)]
mod tests {
    use crate::repositories::order::MockOrderRepository;

    #[tokio::test]
    async fn create_order_success() {
        let mut repo = MockOrderRepository::new();
        repo.expect_create().returning(|_| Ok(order));
        
        let service = OrderService::new(repo);
        let result = service.create_order(input).await;
        
        assert!(result.is_ok());
    }
}
```

**Use `mockall`** for repository mocks (already in workspace).

## Module Registration

Add to `src/lib.rs`:
```rust
pub mod service;
```

Add to `src/service/mod.rs`:
```rust
pub mod order;  // your new service
```

## Checklist for New Services

- [ ] Service struct generic over repository trait
- [ ] `new(repo)` constructor
- [ ] Methods take `self` (not `&mut self`)
- [ ] Return `Result<T, StatusCode>`
- [ ] Delegate validation/hashing/JWT to `auth::validation`
- [ ] Integration tests with `MockRepository`
- [ ] Export in `src/service/mod.rs`
- [ ] Register in `src/lib.rs`
- [ ] Update handlers to use service