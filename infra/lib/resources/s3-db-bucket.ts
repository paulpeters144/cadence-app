import * as cdk from "aws-cdk-lib";
import type { Construct } from "constructs";

interface S3DbBucketProps {
  construct: Construct;
  appName: string;
  stage: string;
}

export class S3DbBucket {
  resource: cdk.aws_s3.Bucket;

  constructor({ construct, appName, stage }: S3DbBucketProps) {
    const bucketName = `${appName}-${stage}-db-bucket`;
    this.resource = new cdk.aws_s3.Bucket(construct, bucketName, {
      bucketName: bucketName,
      removalPolicy: cdk.RemovalPolicy.RETAIN, // Keep DB files even if stack is deleted
      versioned: true, // Good for DB files
      encryption: cdk.aws_s3.BucketEncryption.S3_MANAGED,
      blockPublicAccess: cdk.aws_s3.BlockPublicAccess.BLOCK_ALL,
    });
  }
}
