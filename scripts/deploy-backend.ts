import { execSync } from 'child_process';
import * as path from 'path';
import * as fs from 'fs';
import * as dotenv from 'dotenv';
import { LambdaClient, UpdateFunctionCodeCommand, GetFunctionCommand } from '@aws-sdk/client-lambda';

// Load .env from the root
dotenv.config({ path: path.resolve(process.cwd(), '.env') });

const FUNCTION_NAME = process.env.BACKEND_LAMBDA_FUNCTION_NAME;
const REGION = process.env.AWS_REGION || 'us-east-2';

if (!FUNCTION_NAME) {
  console.error("❌ BACKEND_LAMBDA_FUNCTION_NAME is not defined in the .env file.");
  process.exit(1);
}

const backendDir = path.resolve(process.cwd(), 'backend');
const zipPath = path.join(backendDir, 'target', 'lambda', 'backend', 'bootstrap.zip');

async function main() {
  console.log('📦 Building backend for AWS Lambda...');
  execSync('just build-aws-lambda', { stdio: 'inherit', cwd: backendDir });

  console.log(`\n☁️ Deploying to AWS Lambda Function: ${FUNCTION_NAME}...`);
  const lambdaClient = new LambdaClient({ region: REGION });
  
  if (!fs.existsSync(zipPath)) {
    console.error(`❌ Zip file not found at ${zipPath}`);
    process.exit(1);
  }
  
  const zipBuffer = fs.readFileSync(zipPath);

  const command = new UpdateFunctionCodeCommand({
    FunctionName: FUNCTION_NAME,
    ZipFile: zipBuffer,
  });

  await lambdaClient.send(command);
  
  console.log('⏳ Waiting for function update to complete...');
  let status = 'InProgress';
  while (status === 'InProgress') {
    const response = await lambdaClient.send(new GetFunctionCommand({ FunctionName: FUNCTION_NAME }));
    status = response.Configuration?.LastUpdateStatus || 'Successful';
    if (status === 'InProgress') {
      await new Promise(resolve => setTimeout(resolve, 2000));
    } else if (status === 'Failed') {
      throw new Error(`Function update failed: ${response.Configuration?.LastUpdateStatusReason}`);
    }
  }

  console.log('✅ AWS Lambda deployment finished successfully!');
}

main().catch((err) => {
  console.error('❌ Deployment failed:', err);
  process.exit(1);
});
