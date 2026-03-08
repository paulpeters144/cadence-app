# Cadence Backend

The backend for Cadence is a high-performance REST API built with Rust and Axum. It uses LibSQL (Turso) for data persistence and supports both local development and AWS Lambda deployment.

## Tech Stack

- **Framework**: [Axum](https://github.com/tokio-rs/axum)
- **Database**: [LibSQL](https://libsql.org/) (Turso)
- **Authentication**: JWT (Argon2 for password hashing)
- **Documentation**: [Utoipa](https://github.com/juhakivekas/utoipa) (OpenAPI/Swagger UI)
- **Deployment**: [Lambda Web Adapter](https://github.com/awslabs/aws-lambda-web-adapter) / `lambda-http`

## Getting Started

### Prerequisites

- Rust (latest stable)
- A Turso database or local LibSQL file

### Environment Variables

Create a `.env` file in this directory:

```env
JWT_SECRET=your_super_secret_key
TURSO_DATABASE_URL=file:local.db # or libsql://your-db.turso.io
TURSO_AUTH_TOKEN=your_token # required for remote Turso
```

### Running Locally

```powershell
cargo run
```
The API will be available at `http://localhost:3001`.
Swagger UI is available at `http://localhost:3001/swagger-ui`.

### Testing

```powershell
cargo test
```

## API Documentation

The backend automatically generates an OpenAPI specification. You can export it using:

```powershell
cargo run --bin export_openapi
```
This is used by the frontend to generate TypeScript types.
