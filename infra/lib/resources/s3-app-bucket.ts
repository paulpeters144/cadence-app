import * as cdk from "aws-cdk-lib";
import type { Construct } from "constructs";

interface S3WebAppProps {
  construct: Construct;
  appName: string;
  stage: string;
}

export class S3WebApp {
  resource: cdk.aws_s3.Bucket;

  constructor({ construct, appName, stage }: S3WebAppProps) {
    const bucketName = `${appName}-${stage}-web-app`;
    this.resource = new cdk.aws_s3.Bucket(construct, bucketName, {
      bucketName,
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      autoDeleteObjects: true,
      blockPublicAccess: cdk.aws_s3.BlockPublicAccess.BLOCK_ALL,
      enforceSSL: true,
    });

    const deploymentName = `${appName}-${stage}-web-deployment`;
    new cdk.aws_s3_deployment.BucketDeployment(construct, deploymentName, {
      sources: [cdk.aws_s3_deployment.Source.asset("../frontend/dist")],
      destinationBucket: this.resource,
    });
  }
}
