# Web Request Facade Guidelines

## Overview

This document provides a comprehensive guide for creating **facades** for the web request service. A facade is a wrapper service that extends, limits, or transforms the behavior of a parent web client service while maintaining API compatibility with the underlying system.

## What is a Facade?

A facade in this system:
1. Queries the service tree for a parent web client service
2. Wraps that parent service with custom behavior (rate limiting, logging, request transformation, etc.)
3. Presents the same `tower::Service` API to downstream consumers
4. Supports hot-reload through the reconfigurable service pattern

## When to Use Facades

### Use a Facade When:
- You need to wrap an existing service registered in the tree
- You require hot-reload/reconfiguration support
- You want to compose multiple service behaviors dynamically
- Parent service selection depends on runtime configuration
- You need to maintain the reconfigurable service lifecycle

### Use Tower Middleware When:
- You have simple request/response transformations
- No tree lookup or dynamic parent selection needed
- Acceptable to wire up at compile time
- Don't need independent reconfiguration lifecycle
- Want minimal overhead (no background tasks/channels)

**Rule of thumb**: Facades are for dynamic, tree-integrated service composition. Middleware is for static transformation layers.

## Reference: Core Types and Locations

This section lists key types and their source files for easy reference.

### Type Definitions

| Type | File Location | Description |
|------|---------------|-------------|
| `RegisteredServiceTree` | `src/adapters/service_tree.rs` | Global service registry tree |
| `ServiceManagement` | `src/adapters/service_kind.rs` | Tagged union of all service types |
| `ReconfigurableService<C, Req, Res>` | `src/adapters/reconfigurable.rs` | Hot-reload wrapper for services |
| `RequestHandle<C, Req, Res>` | `src/adapters/reconfigurable.rs` | Handle for making requests to reconfigurable services |
| `Reconfig<Req, Res>` | `src/adapters/reconfigurable.rs` | Trait for reconfigurable services |
| `BoxCloneSyncService<Req, Res, E>` | `src/adapters/s3service.rs` | Type-erased clonable service |
| `WebClientService` | `src/services/web_request/service_kind.rs` | Main web client service type |
| `ClientConfig` | `src/services/web_request/config/mod.rs` | Configuration for web client |
| `MaybeFuture<O>` | `src/adapters/maybe_async.rs` | Either sync or async result |
| `BoxedConfig` | `src/traits/mod.rs` | Type-erased configuration (`Box<dyn Any + Send>`) |
| `reqwest::Request` | External crate | HTTP request type (reqwest) |
| `reqwest::Response` | External crate | HTTP response type (reqwest) |

### Helper Functions

| Function | File Location | Description |
|----------|---------------|-------------|
| `get_tree()` | `src/adapters/service_tree.rs` | Get global service tree singleton |
| `path_relocate(&Vec<String>)` | `src/adapters/path_helper.rs` | Convert Vec<String> to Vec<&str> for tree paths |
| `type_error::<T>()` | `src/traits/errors.rs` | Create type mismatch error |
| `no_such_service(&[&str])` | `src/traits/errors.rs` | Create service not found error |
| `service_has_stopped(&str)` | `src/traits/errors.rs` | Create service stopped error |

## Core Concepts

### Understanding MaybeFuture

`MaybeFuture<O>` is a critical type in this codebase. It's defined as:

```rust
pub type MaybeFuture<O> = Either<Ready<O>, Pin<Box<dyn Future<Output=O> + 'static + Send>>>;
```

This type represents a computation that **may or may not** require async execution:
- `Either::Left(Ready<O>)`: The result is immediately available (sync path)
- `Either::Right(Future)`: The result requires async computation (async path)

**Why it exists**: The service tree uses `Arc<RwLock<ServiceManagement>>`. When the lock is uncontended, access is synchronous. When contended, it becomes async. This optimization avoids unnecessary async overhead.

**How to use it**: MaybeFuture implements the `Future` trait, so just `.await` it:

```rust
// This works for both sync and async cases
let parent = tree.get_service_exact(&path, |mgmt| mgmt.get_web_client()).await?;
```

**DO NOT** try to pattern match or unwrap MaybeFuture manually. Always `.await` it.

### Understanding BoxedConfig

`BoxedConfig` is a type alias for `Box<dyn Any + Send>`. It allows passing arbitrary configuration types through generic interfaces.

To use a boxed config:
```rust
pub async fn reload(&self, config: BoxedConfig) -> anyhow::Result<()> {
    let my_config = config.downcast::<MyFacadeConfig>()
        .map_err(|_| type_error::<MyFacadeConfig>())?;
    self.service.reconfigure(*my_config).await
}
```

### Understanding ReconfigurableService Lifecycle

When you call `service.reconfigure(new_config)`:
1. The new config is sent to the background task
2. The factory is called again with the new config
3. A new service instance is created
4. The old service is replaced
5. A ready check confirms the new service works

**For facades**: This means when your facade is reconfigured, it re-queries the tree for the parent service, getting a fresh reference.

### Understanding BoxCloneSyncService

`BoxCloneSyncService<T, U, E>` is a type-erased, cloneable service wrapper that facades use to hold references to parent services.

**How facades obtain it:**
1. Query the service tree for `ServiceManagement`
2. Call the appropriate getter method (e.g., `get_web_client()`)
3. Receive `BoxCloneSyncService<Request, Response, Error>`

**Key properties:**
- **Always current**: Via the channel architecture, it automatically routes requests to the current parent service instance, even after parent reconfiguration
- **Cheaply cloneable**: Clone on each request to satisfy `async move` ownership requirements
- **Type-erased**: Hides the underlying `RequestHandle` and reconfigurable service implementation

**What facades should NOT do:**
- ❌ Store or own `ServiceManagement` directly
- ❌ Store or own the service tree
- ❌ Query the tree on a per-request basis
- ❌ Try to detect or react to parent reconfigurations

**What facades SHOULD do:**
- ✅ Obtain `BoxCloneSyncService` once during initialization (in the factory)
- ✅ Store it in the wrapper struct
- ✅ Clone it for each request: `let mut parent = self.parent.clone();`
- ✅ Trust the channel architecture to handle parent updates automatically

**Pattern:**
```rust
// In factory - get parent once
let parent = tree.get_service_exact(&path, |mgmt| mgmt.get_web_client()).await?;
Ok(MyFacadeWrapper { parent, config })

// In wrapper's call() - clone for each request
fn call(&mut self, request: Request) -> Self::Future {
    let mut parent = self.parent.clone();  // Cheap clone
    Box::pin(async move {
        parent.call(request).await
    })
}
```

## Core Architecture Components

### Service Tree (`src/adapters/service_tree.rs`)
- **RegisteredServiceTree**: Global tree storing all services at hierarchical paths
- Services accessed via `get_service(&[&str], func)` or `get_service_exact(&[&str], func)`
- Returns `MaybeFuture<Result<O, anyhow::Error>>` - must be `.await`-ed
- `get_service` walks up the tree to find parent paths
- `get_service_exact` only matches the exact path
- Each service wrapped in `Arc<RwLock<ServiceManagement>>`

### Service Management (`src/adapters/service_kind.rs`)
- Tagged union enum for different service types
- `ServiceManagement::WebClient(WebClientService)` is relevant for web facades
- Provides `get_web_client()` to extract the underlying HTTP client
- Returns `BoxCloneSyncService<reqwest::Request, reqwest::Response, anyhow::Error>`
- Each variant has a corresponding getter method

### Reconfigurable Services (`src/adapters/reconfigurable.rs`)
- `ReconfigurableService<C, Req, Res>`: Core hot-reload abstraction
- Type parameters:
  - `C`: Configuration type (must be `Clone + PartialEq + Send + Sync + 'static`)
  - `Req`: Request type (`Send + 'static`)
  - `Res`: Response type (`Send + 'static`)
- Spawns background task handling requests and reconfigurations
- Implements `Reconfig<Req, Res>` trait for config updates
- Factory is called each time config changes
- Factory's Future is `.await`-ed in the background task (see line 228 in reconfigurable.rs)

### Web Client Service (`src/services/web_request/service_kind.rs`)
- Wraps `ReconfigurableService<ClientConfig, reqwest::Request, reqwest::Response>`
- Provides service handles implementing `tower::Service`
- `ReqwestServiceHandle`: Works with `reqwest::{Request, Response}`
- `AxumServiceHandle`: Converts between axum and reqwest types

## Directory Structure for Facades

Each facade should follow this structure:

```
facades/
├── WEB_REQUEST_FACADE_GUIDELINES.md  (this file)
├── mod.rs                             (facade exports)
└── <facade_name>/                     (e.g., rate_limiter, logger, cache)
    ├── mod.rs                         (module exports)
    ├── config.rs                      (configuration types)
    ├── kinds.rs                       (service handle types)
    └── impl.rs                        (core implementation)
```

## Step-by-Step Facade Creation

### Step 1: Define Configuration (`config.rs`)

Your configuration must:
- Derive `Clone`, `PartialEq`, `Send`, `Sync`
- Optionally derive `Serialize`, `Deserialize` for persistence
- Include a reference to the parent service path
- Contain facade-specific settings

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct MyFacadeConfig {
    pub parent_path: Vec<String>,
    pub my_setting: String,
    pub enabled: bool,
}

impl Default for MyFacadeConfig {
    fn default() -> Self {
        Self {
            parent_path: vec!["client".to_string()],
            my_setting: "default_value".to_string(),
            enabled: true,
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct MyFacadeLoader {
    pub path: Vec<String>,
    pub buffer: usize,
    pub config: MyFacadeConfig,
}

impl Default for MyFacadeLoader {
    fn default() -> Self {
        Self {
            path: vec!["my_facade".to_string()],
            buffer: 1024,
            config: MyFacadeConfig::default(),
        }
    }
}
```

### Step 2: Implement Core Service (`impl.rs`)

The implementation should:
1. Query the service tree for the parent (asynchronously)
2. Wrap the parent with custom logic
3. Create an async factory for `ReconfigurableService`

**CRITICAL**: The factory must return an async Future, not `Ready`, because tree queries return `MaybeFuture`.

```rust
use std::{
    future::Future,
    pin::Pin,
};
use tower::Service;
use reqwest::{Request as ReqwestRequest, Response as ReqwestResponse};
use anyhow::anyhow;

use crate::{
    adapters::{
        reconfigurable::ReconfigurableService,
        service_tree::RegisteredServiceTree,
        service_kind::ServiceManagement,
        s3service::BoxCloneSyncService,
        path_helper::path_relocate,
    },
};
use super::config::{MyFacadeConfig, MyFacadeLoader};
use super::kinds::MyFacadeService;

pub fn load_facade(
    tree: RegisteredServiceTree,
    loader: MyFacadeLoader,
) {
    let path = loader.path;
    let reconfigurable = create_reconfigurable(tree.clone(), loader.buffer, loader.config);
    let facade_service = MyFacadeService::new(reconfigurable);
    let manager = ServiceManagement::from(facade_service);
    let path_vec = path_relocate(&path);
    tree.insert(&path_vec, manager);
}

fn create_reconfigurable(
    tree: RegisteredServiceTree,
    buffer: usize,
    config: MyFacadeConfig,
) -> ReconfigurableService<MyFacadeConfig, ReqwestRequest, ReqwestResponse> {
    let factory = MyFacadeFactory { tree };
    ReconfigurableService::new(config, buffer, factory)
}

struct MyFacadeFactory {
    tree: RegisteredServiceTree,
}

impl Clone for MyFacadeFactory {
    fn clone(&self) -> Self {
        Self { tree: self.tree.clone() }
    }
}

impl Service<MyFacadeConfig> for MyFacadeFactory {
    type Response = MyFacadeWrapper;
    type Error = anyhow::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, config: MyFacadeConfig) -> Self::Future {
        let tree = self.tree.clone();
        Box::pin(async move {
            let path: Vec<&str> = config.parent_path.iter().map(|s| s.as_str()).collect();

            let parent = tree.get_service_exact(&path, |mgmt| mgmt.get_web_client()).await?;

            Ok(MyFacadeWrapper {
                parent,
                config,
            })
        })
    }
}

pub struct MyFacadeWrapper {
    parent: BoxCloneSyncService<ReqwestRequest, ReqwestResponse, anyhow::Error>,
    config: MyFacadeConfig,
}

impl Service<ReqwestRequest> for MyFacadeWrapper {
    type Response = ReqwestResponse;
    type Error = anyhow::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.parent.poll_ready(cx)
    }

    fn call(&mut self, mut request: ReqwestRequest) -> Self::Future {
        if self.config.enabled {
            request.headers_mut().insert(
                "X-My-Facade",
                self.config.my_setting.parse().unwrap()
            );
        }

        let mut parent = self.parent.clone();
        Box::pin(async move {
            parent.call(request).await
        })
    }
}
```

**Key points:**
- Factory's `Future` type is `Pin<Box<dyn Future<...>>>`, NOT `Ready`
- Factory's `call()` returns `Box::pin(async move { ... })`
- Tree query is `.await`-ed inside the async block
- Each reconfiguration triggers a new tree query, getting fresh parent reference

### Step 3: Create Service Handles (`kinds.rs`)

Service handles provide type-safe access to your facade:

```rust
use std::pin::Pin;
use std::future::Future;
use tower::Service;
use reqwest::{Request as ReqwestRequest, Response as ReqwestResponse};

use crate::{
    adapters::reconfigurable::{RequestHandle, ReconfigurableService},
    traits::{BoxedConfig, type_error},
};
use super::config::MyFacadeConfig;

pub struct MyFacadeService {
    service: ReconfigurableService<MyFacadeConfig, ReqwestRequest, ReqwestResponse>,
}

impl MyFacadeService {
    pub fn new(service: ReconfigurableService<MyFacadeConfig, ReqwestRequest, ReqwestResponse>) -> Self {
        Self { service }
    }

    pub async fn reload(&self, config: BoxedConfig) -> anyhow::Result<()> {
        let facade_config = config.downcast::<MyFacadeConfig>()
            .map_err(|_| type_error::<MyFacadeConfig>())?;
        self.service.reconfigure(*facade_config).await
    }

    pub fn make_reqwest_service(&self) -> MyFacadeHandle {
        MyFacadeHandle {
            handle: self.service.make_request_handle(),
        }
    }
}

pub struct MyFacadeHandle {
    handle: RequestHandle<MyFacadeConfig, ReqwestRequest, ReqwestResponse>,
}

impl Clone for MyFacadeHandle {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

impl Service<ReqwestRequest> for MyFacadeHandle {
    type Response = ReqwestResponse;
    type Error = anyhow::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.handle.poll_ready(cx)
    }

    fn call(&mut self, req: ReqwestRequest) -> Self::Future {
        self.handle.call(req)
    }
}
```

### Step 4: Module Exports (`mod.rs`)

```rust
pub mod config;
pub mod impl_;
pub mod kinds;

pub use config::{MyFacadeConfig, MyFacadeLoader};
pub use impl_::load_facade;
pub use kinds::{MyFacadeService, MyFacadeHandle};
```

## Integration with ServiceManagement

To integrate your facade with `ServiceManagement` (in `src/adapters/service_kind.rs`):

**When to add to ServiceManagement:**
- If other services need to query for your specific facade type
- If you want type-safe extraction from the tree
- If your facade should be a first-class service type

**When NOT to add:**
- If your facade is only accessed by path (consumers don't need to know it's a facade)
- If it's always used as a generic WebClient-compatible service
- If you want to keep the facade implementation private

**How to add (if needed):**

```rust
// In src/adapters/service_kind.rs

// 1. Add variant to enum
#[non_exhaustive]
pub enum ServiceManagement {
    WebClient(WebClientService),
    OpenRouter(Box<dyn Reconfig<ORRequest, ORResponse> + 'static>),
    EndPoint(Box<dyn Reconfig<ExtHttpRequest, ExtHttpResponse> + 'static>),
    MyFacade(MyFacadeService),  // Your facade
}

// 2. Add From implementation
impl From<MyFacadeService> for ServiceManagement {
    fn from(item: MyFacadeService) -> Self {
        Self::MyFacade(item)
    }
}

// 3. Add getter method
impl ServiceManagement {
    pub fn get_my_facade(&self) -> Result<BoxCloneSyncService<ReqwestRequest, ReqwestResponse, anyhow::Error>, anyhow::Error> {
        match self {
            Self::MyFacade(facade) => Ok(BoxCloneSyncService::new(facade.make_reqwest_service())),
            _ => Err(not_an_http_client::<Self>()),
        }
    }
}

// 4. Update reload() to handle your variant
impl ServiceManagement {
    pub async fn reload(&self, config: BoxedConfig) -> Result<(), anyhow::Error> {
        match self {
            Self::WebClient(client) => client.reload(config).await,
            Self::OpenRouter(client) => client.reconfig(config).await,
            Self::EndPoint(client) => client.reconfig(config).await,
            Self::MyFacade(facade) => facade.reload(config).await,  // Add this
        }
    }
}
```

## Facade Lifecycle and Parent Reconfiguration

### Understanding the Channel-Based Architecture

This is critical to understand: `ReconfigurableService` uses a **channel-based architecture** where all service handles share the same background task.

**Key components:**
- `ReconfigurableService` spawns a background task with a channel receiver
- `RequestHandle` holds a channel sender (cloned from the parent)
- All requests go through the channel to the background task
- The background task holds the actual service instance

### What Happens When Parent is Reconfigured?

Scenario: Parent service at `/client` is reconfigured via `tree.reload(&["client"], new_config)`

**Step-by-step:**
1. Parent's background task receives `Reconfigure` message on the channel
2. Factory is called to create a **new service instance**
3. Background task switches to using the new instance
4. **All existing RequestHandles automatically route to the new instance** through the same channel

**Critical insight**: Facades holding `BoxCloneSyncService<Request, Response, Error>` (which wraps `RequestHandle`) **automatically pick up the reconfigured parent service**. No action needed.

From `reconfigurable.rs:223-229`:
```rust
ServiceComms::Reconfigure(new_config) => {
    config.clone_from(&new_config);
    service = Some(factory.call(new_config).await...);
    continue 'reload;  // Uses NEW service for ALL subsequent requests
}
```

### What Happens When Facade is Reconfigured?

Scenario: Facade is reconfigured via `tree.reload(&["my_facade"], new_config)`

**Step-by-step:**
1. Facade's factory is called with new config
2. Factory queries tree for parent (gets current `RequestHandle`)
3. Facade wraps the parent
4. Facade's background task switches to new wrapper instance

**When to reconfigure facade:**
- Facade's own settings changed (e.g., rate limit, custom headers)
- Parent path changed (switching to different parent)
- **NOT** needed just because parent was reconfigured (automatic via channel)

### Channel Architecture Diagram

```
Facade Wrapper
    └─> BoxCloneSyncService
        └─> RequestHandle (holds channel sender)
            └─> [Channel]
                └─> Parent's Background Task
                    └─> Current Service Instance (updated on reconfig)
```

When parent reconfigures, only the "Current Service Instance" changes. The entire chain above it remains the same and continues working.

### Handling Parent Service Removal

If the parent service is **removed from the tree** (not reconfigured):
- Facade continues working with its cached `RequestHandle`
- Parent's background task still runs
- Requests continue to work normally

However, **new facade configurations** will fail to find the parent:
```rust
// In factory - this will error if parent was removed
let parent = tree.get_service_exact(&path, |mgmt| mgmt.get_web_client())
    .await
    .map_err(|e| anyhow!("Parent service '{}' not found: {}", config.parent_path.join("/"), e))?;
```

### When Facades Actually Break

Facades break in these scenarios (NOT during parent reconfiguration):

**1. Parent Service Stopped**
- Parent's background task is stopped via `request_immediate_stop()` or `request_graceful_stop()`
- All RequestHandles return "service has stopped" errors
- Facade requests fail immediately

**2. Parent Service Misconfigured**
- Parent factory returns an error during reconfiguration
- Background task enters error state
- Facade requests fail with "service is misconfigured" error

**3. Parent Service Removed + New Facade Config**
- Parent removed from tree
- Attempt to reconfigure or create new facade instance
- Factory can't find parent, returns error

**4. Channel Closed**
- Parent service panics or is dropped
- Channel is closed
- Facade requests fail with channel errors

**What does NOT break facades:**
- ✅ Parent service reconfiguration (automatic via channel)
- ✅ Parent config changes (propagated automatically)
- ✅ Multiple parent reconfigurations (all automatic)

### Best Practices

**DO:**
- Store `BoxCloneSyncService` reference (as shown in examples)
- Trust the channel architecture to propagate parent changes
- Only reconfigure facade when facade's own config changes

**DON'T:**
- Re-query tree on every request (unnecessary overhead)
- Reconfigure facade just because parent was reconfigured
- Try to detect parent config changes (automatic)

**Key takeaway**: The channel-based architecture means parent reconfiguration is **transparent** to facades. This is by design and a key benefit of the `ReconfigurableService` pattern.

## Common Patterns

### Pattern 1: Request Transformation
Modify requests before forwarding to parent:
```rust
fn call(&mut self, mut request: ReqwestRequest) -> Self::Future {
    request.headers_mut().insert(
        "X-Custom-Header",
        "my-value".parse().unwrap()
    );

    let mut parent = self.parent.clone();
    Box::pin(async move {
        parent.call(request).await
    })
}
```

### Pattern 2: Response Transformation
Modify responses from parent:
```rust
fn call(&mut self, request: ReqwestRequest) -> Self::Future {
    let mut parent = self.parent.clone();
    Box::pin(async move {
        let mut response = parent.call(request).await?;
        response.headers_mut().insert(
            "X-Processed-By",
            "my-facade".parse().unwrap()
        );
        Ok(response)
    })
}
```

### Pattern 3: Rate Limiting
Control request rate:
```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct RateLimitWrapper {
    parent: BoxCloneSyncService<ReqwestRequest, ReqwestResponse, anyhow::Error>,
    semaphore: Arc<Semaphore>,
}

impl Service<ReqwestRequest> for RateLimitWrapper {
    type Response = ReqwestResponse;
    type Error = anyhow::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.parent.poll_ready(cx)
    }

    fn call(&mut self, request: ReqwestRequest) -> Self::Future {
        let semaphore = self.semaphore.clone();
        let mut parent = self.parent.clone();
        Box::pin(async move {
            let _permit = semaphore.acquire().await.unwrap();
            parent.call(request).await
        })
    }
}
```

### Pattern 4: Conditional Routing
Route to different backends based on request properties:
```rust
pub struct RoutingWrapper {
    default_parent: BoxCloneSyncService<ReqwestRequest, ReqwestResponse, anyhow::Error>,
    alternative_parent: BoxCloneSyncService<ReqwestRequest, ReqwestResponse, anyhow::Error>,
}

impl Service<ReqwestRequest> for RoutingWrapper {
    type Response = ReqwestResponse;
    type Error = anyhow::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: ReqwestRequest) -> Self::Future {
        let use_alternative = request.url().host_str() == Some("special.example.com");
        let mut service = if use_alternative {
            self.alternative_parent.clone()
        } else {
            self.default_parent.clone()
        };

        Box::pin(async move {
            service.call(request).await
        })
    }
}
```

### Pattern 5: Logging/Observability
Log requests and responses:
```rust
fn call(&mut self, request: ReqwestRequest) -> Self::Future {
    let url = request.url().clone();
    let method = request.method().clone();

    let mut parent = self.parent.clone();
    Box::pin(async move {
        tracing::info!("Request: {} {}", method, url);
        let start = std::time::Instant::now();

        let result = parent.call(request).await;

        let duration = start.elapsed();
        match &result {
            Ok(resp) => tracing::info!("Response: {} {} - {} ({:?})", method, url, resp.status(), duration),
            Err(e) => tracing::error!("Error: {} {} - {} ({:?})", method, url, e, duration),
        }

        result
    })
}
```

## Testing Facades

Create tests in your `impl.rs` or separate test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::service_tree::get_tree;
    use crate::services::web_request::config::ClientLoader;
    use crate::services::web_request::service_impl::load_default_client;

    #[tokio::test]
    async fn test_facade_wraps_parent() {
        let tree = get_tree();

        let parent_config = ClientLoader::default();
        load_default_client(tree.clone(), parent_config);

        let facade_loader = MyFacadeLoader::default();
        load_facade(tree.clone(), facade_loader);

        let facade_result = tree.get_service_exact(
            &["my_facade"],
            |mgmt| mgmt.get_my_facade()
        ).await;

        assert!(facade_result.is_ok());
    }

    #[tokio::test]
    async fn test_facade_reconfiguration() {
        let tree = get_tree();

        let parent_config = ClientLoader::default();
        load_default_client(tree.clone(), parent_config);

        let facade_loader = MyFacadeLoader::default();
        load_facade(tree.clone(), facade_loader);

        let new_config = MyFacadeConfig {
            parent_path: vec!["client".to_string()],
            my_setting: "new_value".to_string(),
            enabled: false,
        };

        let reload_result = tree.reload(
            &["my_facade"],
            Box::new(new_config)
        ).await;

        assert!(reload_result.is_ok());
    }
}
```

## Key Considerations

1. **Error Handling**: Always propagate errors from parent services properly
2. **Clone Semantics**: Services must be cheaply cloneable (use Arc for shared state)
3. **Async Safety**: All futures must be `Send + 'static`
4. **Configuration Changes**: Facade reconfiguration queries tree for fresh parent handle; parent reconfigurations propagate automatically via channel
5. **Performance**: Minimize overhead in wrapper's `call()` method
6. **Type Compatibility**: Maintain `tower::Service` trait compatibility
7. **Resource Cleanup**: Reconfigurable services handle cleanup automatically
8. **Parent Lifetime**: Parent handles remain valid via channel even if parent is removed from tree
9. **MaybeFuture**: Always `.await` tree queries, never try to unwrap manually
10. **Factory Futures**: Must be async (`Pin<Box<dyn Future>>`) if querying tree
11. **Channel Architecture**: Parent reconfiguration is transparent to facades - requests automatically route to current service instance

## Required Dependencies

Add to your facade's module:

```rust
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::Service;
use reqwest::{Request as ReqwestRequest, Response as ReqwestResponse};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::{
    adapters::{
        reconfigurable::{ReconfigurableService, RequestHandle},
        service_tree::RegisteredServiceTree,
        service_kind::ServiceManagement,
        s3service::BoxCloneSyncService,
        path_helper::path_relocate,
    },
    traits::{BoxedConfig, type_error},
};
```

## Example Use Cases

### Rate Limiting Facade
Limits request rate to external APIs to prevent throttling. Uses `tokio::sync::Semaphore` to control concurrency.

### Logging Facade
Logs all requests/responses for debugging and monitoring. Uses `tracing` crate for structured logging.

### Caching Facade
Caches responses based on request patterns. Uses `moka` or similar cache implementation.

### Circuit Breaker Facade
Implements circuit breaker pattern for resilient external calls. Tracks failures and opens circuit when threshold exceeded.

### Retry Facade
Automatically retries failed requests with exponential backoff. Uses `tokio::time::sleep` between attempts.

### Authentication Facade
Automatically adds authentication headers to all requests. Manages token refresh and credential storage.

### Request Validation Facade
Validates requests before forwarding to prevent invalid API calls. Checks headers, body structure, etc.

### Load Balancing Facade
Distributes requests across multiple parent services. Can wrap multiple parent paths and route based on strategy.

### Metrics Facade
Collects request/response metrics for monitoring. Tracks latency, status codes, error rates.

### Header Injection Facade
Adds common headers to all requests (User-Agent, API keys, correlation IDs, etc.).

## Debugging Tips

1. **Service Tree Issues**: Use `tree.contains_path(&path)` to verify parent exists before loading facade
2. **Configuration Errors**: Implement `Debug` for all config types; errors show type names
3. **Async Issues**: Ensure all futures are properly pinned and bounded with `Send + 'static`
4. **Type Errors**: Use `std::any::type_name::<T>()` for better error messages in downcasting
5. **Deadlocks**: Never hold locks across `.await` points
6. **MaybeFuture confusion**: If you see "expected Future, found MaybeFuture" - just `.await` it
7. **Factory not async**: If factory won't compile, ensure `Future` type is `Pin<Box<dyn Future>>` not `Ready`
8. **Parent not found**: Check parent path exists in tree before facade loads
9. **Parent reconfiguration**: Facades automatically pick up parent changes via channel - no action needed
10. **Service stopped errors**: Check if parent service task has crashed or been stopped (not the same as reconfigured)

## Summary

Creating a facade involves:
1. Define configuration with parent service path
2. Implement async factory that queries tree and wraps parent
3. Create wrapper service implementing custom logic
4. Provide typed service handles
5. Export via mod.rs
6. Optionally integrate with ServiceManagement enum

**Critical reminders**:
- Factory must return `Pin<Box<dyn Future>>`, not `Ready`
- Always `.await` MaybeFuture results from tree queries
- Facade reconfiguration re-queries tree; parent reconfigurations are automatic via channel
- Parent reconfiguration is transparent - facades automatically use updated parent service
- Use facades for dynamic composition, middleware for static transformation

Follow these guidelines to create composable, reloadable service facades that extend the web request system's capabilities.
