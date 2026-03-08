# AWS Lambda & Axum Hybrid Runtime Migration Plan

## Project Overview
The objective is to adapt the existing Rust (Axum) backend to run natively on AWS Lambda using the `lambda_http` crate while preserving the ability to run it as a standard local HTTP server (`axum::serve`) for seamless, frictionless local development. The deployment build process will be standardized using `cargo-lambda` (installed via `pip`) to handle cross-compilation from Windows to the Amazon Linux ARM64 environment.

## Technical Stack
- **Language:** Rust (2024 edition)
- **Web Framework:** Axum
- **Lambda Adapter:** `lambda_http`
- **Build Tooling:** `cargo`, `cargo-lambda` (via Python `pip`)
- **Infrastructure:** AWS CDK (existing setup targeting `arm64`)

## Phases of Development

### Phase 1: Tooling Setup (Research & Discovery)
1. **Install `cargo-lambda`:** Ensure `cargo-lambda` is installed globally using Python's package manager. This provides the cross-compilation toolchain required for deploying Windows-built Rust code to AWS Lambda Linux servers.
   - Command: `pip install cargo-lambda`
2. **Verify Installation:** Confirm the tool is available in the system path (`cargo lambda --version`).

### Phase 2: Dependency Configuration (Architecture & Design)
To support dual runtimes (local vs. Lambda), we will introduce a Cargo feature flag. This prevents Lambda-specific dependencies from bloating the local build and vice versa.

1. **Update `Cargo.toml`:**
   - Add `lambda_http` as an optional dependency or standard dependency.
   - Ensure the `tower` dependency has the `make` feature enabled, which is required by `lambda_http`.
   - Introduce a new feature (e.g., `lambda`) to toggle the runtime behavior.

### Phase 3: Implementation Steps (Code Adaptation)
The core change involves modifying `src/main.rs` to conditionally compile either the Lambda runtime or the local TCP listener based on feature flags.

1. **Modify `backend/src/main.rs`:**
   - Implement conditional compilation using `#[cfg(feature = "lambda")]` and `#[cfg(not(feature = "lambda"))]`.
   - **Local Path (`not(feature = "lambda")`):** Retain the existing `tokio::net::TcpListener::bind` and `axum::serve` logic.
   - **Lambda Path (`feature = "lambda"`):** Implement `lambda_http::run(router).await`.
   - Add a tracing subscriber setup specifically for the Lambda path to ensure logs are correctly formatted for AWS CloudWatch.

### Phase 4: Build Script & Workflow Updates
Update the local development scripts to accommodate the dual-runtime setup and ensure the `.env` file is packaged for Lambda.

1. **Update `backend/Justfile`:**
   - Modify the `build-aws-lambda` recipe to explicitly pass the required feature flag (if used) to `cargo lambda build`. For example: `cargo lambda build --release --arm64 --features lambda --output-format binary`.
   - Ensure the `bootstrap` binary is copied to `backend/dist`.
   - **Crucially:** Add a step to copy the `.env` file into `backend/dist` alongside the binary so `dotenvy` can find it at runtime.
     *Example `Justfile` snippet:*
     ```powershell
     if (!(Test-Path dist)) { New-Item -ItemType Directory dist }
     Copy-Item -Path target/lambda/backend/bootstrap -Destination dist/bootstrap -Force
     Copy-Item -Path .env -Destination dist/.env -Force
     ```
   - Ensure the existing `dev` and `watch` recipes continue to run the standard local server without the Lambda adapter.

### Phase 5: Testing & Quality Assurance
1. **Local Server Test:** Run `just dev` or `just watch` and verify the Axum server starts on `0.0.0.0:3001` and serves requests normally.
2. **Lambda Build Test:** Run `just build-aws-lambda`. Verify that:
   - The cross-compilation succeeds without errors.
   - The resulting `bootstrap` binary AND the `.env` file are successfully placed in the `backend/dist` directory.
3. **CDK Deployment Test:** (Optional but recommended) Run the CDK deploy command to push the artifact to AWS and test the live Lambda Function URL.
