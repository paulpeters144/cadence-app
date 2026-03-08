# Cadence

Cadence is a personal productivity application designed for agile-style task management. It features points-based task estimation, sprint (collection) management, performance tracking, and integrated journaling.

## Project Structure

This is a monorepo managed with `pnpm` workspaces:

- `frontend/`: React + TypeScript + Vite + Tailwind CSS (PWA)
- `backend/`: Rust + Axum + LibSQL (Turso)
- `infra/`: AWS CDK (AWS Lambda, API Gateway, S3, CloudFront)
- `scripts/`: Helper scripts for development and deployment

## Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/) (v1.85+)
- [AWS CLI](https://aws.amazon.com/cli/) & [CDK](https://docs.aws.amazon.com/cdk/v2/guide/home.html) (for deployment)

## Quick Start

1. Install all dependencies:
   ```powershell
   pnpm install
   ```

2. Set up environment variables:
   - Create `.env` in `backend/` (see `backend/.env.example` if available)
   - Create `.env` in `infra/` (see `infra/.env.example`)

3. Run the development environment:
   ```powershell
   pnpm run dev
   ```
   This will start both the frontend and backend concurrently.

## Available Scripts

- `pnpm run dev`: Start frontend and backend in development mode.
- `pnpm run build:frontend`: Build the frontend for production.
- `pnpm run generate`: Generate API types from the backend OpenAPI spec.
- `pnpm run deploy:backend`: Build and deploy the backend to AWS Lambda.
- `pnpm run deploy:frontend`: Build and deploy the frontend to AWS S3/CloudFront.

## License

MIT - See [LICENSE](./LICENSE) for details.
