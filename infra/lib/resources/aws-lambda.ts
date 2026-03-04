import * as cdk from "aws-cdk-lib";
import type { Construct } from "constructs";

interface AwsLambdaProps {
  construct: Construct;
  appName: string;
  stage: string;
  dbBucket: cdk.aws_s3.Bucket;
}

export class AwsLambda {
  resource: cdk.aws_lambda.Function;

  constructor(props: AwsLambdaProps) {
    const { construct, appName, stage, dbBucket } = props;
    const apiLambdaName = `${appName}-lambda-${stage}`;
    
    this.resource = new cdk.aws_lambda.Function(construct, apiLambdaName, {
      functionName: apiLambdaName,
      timeout: cdk.Duration.seconds(29), // API Gateway max timeout is 29s
      runtime: cdk.aws_lambda.Runtime.PROVIDED_AL2023,
      code: cdk.aws_lambda.Code.fromAsset("../backend/dist"),
      handler: "bootstrap", // Standard for Rust
      architecture: cdk.aws_lambda.Architecture.ARM_64,
      memorySize: 512,
      environment: {
        DB_BUCKET_NAME: dbBucket.bucketName,
        APP_STAGE: stage,
        // Add other env vars as needed
      },
    });

    // Grant the lambda permissions to read/write from the DB bucket
    dbBucket.grantReadWrite(this.resource);
  }
}
