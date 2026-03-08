import { execSync } from 'child_process';
import * as path from 'path';
import * as fs from 'fs';
import * as dotenv from 'dotenv';
import { LambdaClient, UpdateFunctionCodeCommand } from '@aws-sdk/client-lambda';

// Load .env from the root
dotenv.config({ path: path.resolve(process.cwd(), '.env') });

const FUNCTION_NAME = process.env.BACKEND_LAMBDA_FUNCTION_NAME;
const REGION = process.env.BACKEND_AWS_REGION || 'us-east-2';

if (!FUNCTION_NAME) {
  console.error("❌ BACKEND_LAMBDA_FUNCTION_NAME is not defined in the .env file.");
  process.exit(1);
}

const backendDir = path.resolve(process.cwd(), 'backend');
const distDir = path.join(backendDir, 'dist');
const zipPath = path.join(backendDir, 'lambda.zip');

async function main() {
  console.log('📦 Building backend for AWS Lambda...');
  execSync('just build-aws-lambda', { stdio: 'inherit', cwd: backendDir });

  console.log('🗜️ Zipping payload...');
  // Clean up old zip if it exists
  if (fs.existsSync(zipPath)) fs.rmSync(zipPath);
  
  // Use PowerShell native archiving to zip the contents of dist
  execSync(`powershell.exe -NoProfile -Command "Compress-Archive -Path '${path.join(distDir, '*')}' -DestinationPath '${zipPath}' -Force"`, { stdio: 'inherit' });

  console.log(`\n☁️ Deploying to AWS Lambda Function: ${FUNCTION_NAME}...`);
  const lambdaClient = new LambdaClient({ region: REGION });
  const zipBuffer = fs.readFileSync(zipPath);

  const command = new UpdateFunctionCodeCommand({
    FunctionName: FUNCTION_NAME,
    ZipFile: zipBuffer,
  });

  await lambdaClient.send(command);
  console.log('✅ AWS Lambda deployment finished successfully!');
}

main().catch((err) => {
  console.error('❌ Deployment failed:', err);
  process.exit(1);
});
