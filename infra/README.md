# Cadence App Infrastructure

This project contains the AWS CDK setup for the Cadence App.

## Resources
- **AWS Lambda**: Hosts the Rust Axum API.
- **API Gateway**: Exposes the Lambda function via HTTP.
- **S3 Bucket (Web App)**: Hosts the frontend assets.
- **S3 Bucket (Database)**: Holds the LibSQL database file.
- **CloudFront**: Distributes the frontend assets with HTTPS.

## Commands
- `pnpm run cdk:diff:uat`: See the infrastructure changes for UAT.
- `pnpm run cdk:deploy:uat`: Deploy the infrastructure to UAT.
- `pnpm run cdk:diff:prod`: See the infrastructure changes for Production.
- `pnpm run cdk:deploy:prod`: Deploy the infrastructure to Production.

## Deployment Context
The project uses CDK context to manage stages (`uat`, `production`). Use the `-c stage=...` flag if running CDK commands manually.
