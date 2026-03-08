import { execSync } from 'child_process';
import * as path from 'path';
import * as fs from 'fs';
import * as dotenv from 'dotenv';
import { S3Client } from '@aws-sdk/client-s3';
import { CloudFrontClient, CreateInvalidationCommand } from '@aws-sdk/client-cloudfront';
import { S3SyncClient } from 's3-sync-client';

// Load .env from the root
dotenv.config({ path: path.resolve(process.cwd(), '.env') });

const BUCKET_NAME = process.env.FRONTEND_S3_BUCKET;
const REGION = process.env.AWS_REGION || 'us-east-1';
const CLOUDFRONT_DIST_ID = process.env.CLOUDFRONT_DISTRIBUTION_ID;
const API_URL = process.env.VITE_API_BASE_URL;

if (!BUCKET_NAME) {
  console.error("❌ FRONTEND_S3_BUCKET is not defined in the .env file.");
  process.exit(1);
}

if (!API_URL) {
  console.warn("⚠️ VITE_API_BASE_URL is not defined in the root .env file. The build will use the default or current build-time value.");
}

const frontendDir = path.resolve(process.cwd(), 'frontend');
const distDir = path.join(frontendDir, 'dist');

async function main() {
  console.log('🧹 Cleaning previous build...');
  if (fs.existsSync(distDir)) {
    fs.rmSync(distDir, { recursive: true, force: true });
    console.log('✅ Build directory cleaned.');
  }

  console.log('\n📦 Building frontend...');
  // Execute the frontend build script
  execSync('pnpm --filter frontend build', { 
    stdio: 'inherit', 
    cwd: process.cwd(),
    env: { ...process.env, VITE_API_BASE_URL: API_URL }
  });

  if (!fs.existsSync(distDir)) {
    console.error(`❌ Build directory not found: ${distDir}`);
    process.exit(1);
  }

  console.log(`\n☁️ Deploying to S3 Bucket: ${BUCKET_NAME}...`);
  const s3Client = new S3Client({ region: REGION });
  const syncClient = new S3SyncClient({ client: s3Client });

  // Sync dist directory to S3 (with del: true to remove old files)
  await syncClient.sync(distDir, `s3://${BUCKET_NAME}`, {
    del: true,
  });
  console.log('✅ S3 Sync Complete.');

  // Invalidate CloudFront cache if a distribution ID is provided
  if (CLOUDFRONT_DIST_ID) {
    console.log(`\n🔄 Invalidating CloudFront Cache for ${CLOUDFRONT_DIST_ID}...`);
    const cfClient = new CloudFrontClient({ region: REGION });
    const invalidationParams = {
      DistributionId: CLOUDFRONT_DIST_ID,
      InvalidationBatch: {
        CallerReference: Date.now().toString(),
        Paths: {
          Quantity: 1,
          Items: ['/*'],
        },
      },
    };
    await cfClient.send(new CreateInvalidationCommand(invalidationParams));
    console.log('✅ CloudFront Invalidation Requested.');
  }

  console.log('\n🚀 Deployment finished successfully!');
}

main().catch((err) => {
  console.error('❌ Deployment failed:', err);
  process.exit(1);
});
