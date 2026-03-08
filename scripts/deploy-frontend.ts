import { execSync } from 'child_process';
import * as path from 'path';
import * as fs from 'fs';
import * as dotenv from 'dotenv';
import { S3Client, PutObjectCommand, ListObjectsV2Command, DeleteObjectsCommand } from '@aws-sdk/client-s3';
import { CloudFrontClient, CreateInvalidationCommand } from '@aws-sdk/client-cloudfront';
import * as mime from 'mime-types';

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

const frontendDir = path.resolve(process.cwd(), 'frontend');
const distDir = path.join(frontendDir, 'dist');

async function getAllFiles(dirPath: string, arrayOfFiles: string[] = []) {
  const files = fs.readdirSync(dirPath);

  for (const file of files) {
    if (fs.statSync(path.join(dirPath, file)).isDirectory()) {
      arrayOfFiles = await getAllFiles(path.join(dirPath, file), arrayOfFiles);
    } else {
      arrayOfFiles.push(path.join(dirPath, file));
    }
  }

  return arrayOfFiles;
}

async function emptyS3Bucket(s3Client: S3Client, bucketName: string) {
  const listParams = { Bucket: bucketName };
  const listedObjects = await s3Client.send(new ListObjectsV2Command(listParams));

  if (!listedObjects.Contents || listedObjects.Contents.length === 0) return;

  const deleteParams = {
    Bucket: bucketName,
    Delete: { Objects: listedObjects.Contents.map(({ Key }) => ({ Key: Key! })) }
  };

  await s3Client.send(new DeleteObjectsCommand(deleteParams));

  if (listedObjects.IsTruncated) await emptyS3Bucket(s3Client, bucketName);
}

async function main() {
  console.log('🧹 Cleaning local build directory...');
  if (fs.existsSync(distDir)) {
    fs.rmSync(distDir, { recursive: true, force: true });
    console.log('✅ Local build directory cleaned.');
  }

  console.log('\n📦 Building frontend...');
  execSync('pnpm --filter frontend build', { 
    stdio: 'inherit', 
    cwd: process.cwd(),
    env: { ...process.env, VITE_API_BASE_URL: API_URL }
  });

  if (!fs.existsSync(distDir)) {
    console.error(`❌ Build directory not found: ${distDir}`);
    process.exit(1);
  }

  const s3Client = new S3Client({ region: REGION });

  console.log(`\n🗑️ Emptying S3 Bucket: ${BUCKET_NAME}...`);
  await emptyS3Bucket(s3Client, BUCKET_NAME as string);
  console.log('✅ S3 Bucket emptied.');

  console.log(`\n☁️ Deploying files to S3...`);
  const files = await getAllFiles(distDir);
  
  const authoritativeMimeTypes: Record<string, string> = {
    '.js': 'application/javascript',
    '.mjs': 'application/javascript',
    '.css': 'text/css',
    '.html': 'text/html',
    '.htm': 'text/html',
    '.json': 'application/json',
    '.svg': 'image/svg+xml',
    '.png': 'image/png',
    '.jpg': 'image/jpeg',
    '.jpeg': 'image/jpeg',
    '.webp': 'image/webp',
    '.ico': 'image/x-icon',
    '.txt': 'text/plain',
  };

  for (const file of files) {
    const relativePath = path.relative(distDir, file);
    const s3Key = relativePath.replace(/\\/g, '/');
    const extension = path.extname(s3Key).toLowerCase();
    const contentType = authoritativeMimeTypes[extension] || (mime.lookup(s3Key) as string) || 'application/octet-stream';

    console.log(`📡 Uploading ${s3Key} (${contentType})...`);
    
    const fileContent = fs.readFileSync(file);
    await s3Client.send(new PutObjectCommand({
      Bucket: BUCKET_NAME as string,
      Key: s3Key,
      Body: fileContent,
      ContentType: contentType,
    }));
  }
  console.log('✅ S3 Upload Complete.');

  if (CLOUDFRONT_DIST_ID) {
    console.log(`\n🔄 Invalidating CloudFront Cache for ${CLOUDFRONT_DIST_ID}...`);
    const cfClient = new CloudFrontClient({ region: REGION });
    const invalidationParams = {
      DistributionId: CLOUDFRONT_DIST_ID as string,
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
