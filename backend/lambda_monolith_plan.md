# Lambda Monolith (Lambdalith) Migration Plan

This plan outlines the steps to convert the existing Axum-based backend into a Lambda-compatible monolith using `lambda-http` and connecting to a Turso database.

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
# Ensure tower has required features for service adaptation
tower = { version = "0.5.3", features = ["util", "make"] }
```
*(Note: AWS SDK dependencies for S3 are no longer needed as Turso handles database synchronization.)*

## Phase 3: Lambda Adaptation (`src/main.rs`)

Replace the existing `main` function with a Lambda-compatible one:

1.  **Initialization (Outside `run`)**:
    - Setup Tracing.
    - Initialize `AppRepository` pointing to the Turso database.
    - Initialize `AppManager` and Axum `Router`.

2.  **Lambda Loop**:
    - Use `lambda_http::run(app).await`.
    - `lambda_http::run` takes control of the execution loop to handle incoming Lambda events and pass them to the Axum router.

## Phase 4: Local Development

1.  **Local Emulator**:
    ```powershell
    cargo lambda watch
    ```
2.  **Environment Variables**: Ensure `.env` contains:
    ```env
    TURSO_DATABASE_URL=libsql://your-database-url.turso.io
    TURSO_AUTH_TOKEN=your_auth_token
    ```

## Phase 5: Build & Deployment

1.  **Build**:
    ```powershell
    cargo lambda build --release
    ```
2.  **Deploy**:
    ```powershell
    cargo lambda deploy --enable-function-url backend
    ```
3.  **Infrastructure Configuration**:
    - Set environment variables in AWS Lambda:
      - `TURSO_DATABASE_URL` (Turso connection URL)
      - `TURSO_AUTH_TOKEN`
      - `JWT_SECRET`

## Phase 6: Verification

1.  Test all endpoints via the Lambda Function URL.
2.  Verify that data is correctly persisting to the Turso database.
3.  Check CloudWatch logs for any initialization or connection errors.