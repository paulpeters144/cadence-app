# Cadence App Infrastructure

This project contains the AWS CDK setup for the Cadence App, providing a fully automated deployment pipeline for both the backend and frontend.

## Architecture

- **AWS Lambda**: Hosts the Rust Axum API (packaged as a container or via `cargo-lambda`).
- **API Gateway**: Provides the HTTP interface for the Lambda function.
- **S3 Bucket (Web App)**: Hosts the static React/Vite assets.
- **S3 Bucket (Database)**: Holds the LibSQL database file (for S3-based LibSQL sync).
- **CloudFront**: Global CDN for the frontend assets with SSL/TLS.

## Prerequisites

- AWS CLI configured with appropriate permissions.
- Node.js and `pnpm`.

## Commands

- `pnpm run cdk:diff:uat`: Compare the current state with the deployed UAT stack.
- `pnpm run cdk:deploy:uat`: Deploy the infrastructure to the UAT environment.

## Environment Configuration

Copy `.env.example` to `.env` and fill in the required variables before deploying.

```env
DOMAIN_NAME=yourdomain.com
CERTIFICATE_ARN=arn:aws:acm:...
```

## Deployment Context

The project uses CDK context to manage stages (e.g., `uat`). Use `-c stage=uat` if running CDK commands manually.
