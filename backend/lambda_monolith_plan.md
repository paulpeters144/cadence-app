# Lambda Monolith (Lambdalith) Migration Plan

This plan outlines the steps to convert the existing Axum-based backend into a Lambda-compatible monolith using `lambda-http` and implementing the S3-SQLite synchronization protocol.

## Phase 1: Environment & Tooling

1.  **Install `cargo-lambda`**:
    ```powershell
    # Windows (using pip or scoop)
    pip install cargo-lambda
    # OR
    scoop install cargo-lambda
    ```
2.  **Verify AWS CLI**: Ensure AWS credentials are configured for deployment.

## Phase 2: Dependency Configuration

Update `Cargo.toml` with the following additions:

```toml
[dependencies]
# Existing dependencies...
lambda_http = "0.13"
aws-config = "1.1"
aws-sdk-s3 = "1.1"
# Ensure tower has required features for service adaptation
tower = { version = "0.5.3", features = ["util", "make"] }
```

## Phase 3: State Management & Sync Logic

### 3.1. Implement `is_dirty` in `AppManager`
- Add `is_dirty: Arc<AtomicBool>` to `AppManager` and `Manager` trait (or provide a way to check/reset it).
- Update all mutating methods in `AppManager` (e.g., `create_list`, `update_task`, `register`, etc.) to set `is_dirty` to `true` upon success.

### 3.2. S3-SQLite Sync Module
Create a utility to handle:
- **Ingress**: `GET` SQLite file from S3 to `/tmp/database.db` on cold start or if ETag mismatches.
- **Egress**: `PUT` SQLite file to S3 if `is_dirty` is true.

## Phase 4: Lambda Adaptation (`src/main.rs`)

Replace the existing `main` function with a Lambda-compatible one:

1.  **Initialization (Outside `run`)**:
    - Setup Tracing.
    - Initialize S3 Client.
    - Perform **Ingress Sync**: Fetch the latest SQLite DB from S3 to `/tmp`.
    - Initialize `AppRepository` pointing to `sqlite:/tmp/database.db`.
    - Initialize `AppManager` and Axum `Router`.

2.  **Lambda Loop**:
    - Use `lambda_http::run(app).await`.
    - **Crucial**: Since `lambda_http::run` takes control, we need to wrap the Axum router in a custom `Tower` middleware or use a hook to perform the **Egress Sync** after the response is generated but before the Lambda freezes.
    - *Alternative*: Wrap the `Router` in a layer that checks the `is_dirty` flag and performs S3 upload before returning the response.

## Phase 5: Local Development

1.  **Local Emulator**:
    ```powershell
    cargo lambda watch
    ```
2.  **Mock S3**: For local development, use a local file system or a mock S3 (like MinIO or LocalStack) if full sync testing is needed.

## Phase 6: Build & Deployment

1.  **Build**:
    ```powershell
    cargo lambda build --release
    ```
2.  **Deploy**:
    ```powershell
    cargo lambda deploy --enable-function-url backend
    ```
3.  **Infrastructure Configuration**:
    - Set `DATABASE_URL` to `sqlite:/tmp/database.db`.
    - Set `S3_BUCKET_NAME` and `S3_KEY_NAME`.
    - **IMPORTANT**: Set `Reserved Concurrency = 1` in AWS Lambda settings to prevent race conditions.

## Phase 7: Verification

1.  Test all endpoints via the Lambda Function URL.
2.  Verify that SQLite changes persist across Lambda cold starts by checking the S3 bucket.
3.  Check CloudWatch logs for sync duration and errors.
