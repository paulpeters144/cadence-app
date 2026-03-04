import * as cdk from "aws-cdk-lib";
import type { Construct } from "constructs";
import * as resource from "./resources";

export class InfraStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: cdk.StackProps) {
    super(scope, id, props);

    if (!props?.tags?.stage) {
      throw new Error("The 'stage' tag is required in props.");
    }
    if (!props?.tags?.appName) {
      throw new Error("The 'appName' tag is required in props.");
    }

    const stage = props.tags.stage;
    const appName = props.tags.appName;

    // 1. S3 Bucket for LibSQL file
    const dbBucket = new resource.S3DbBucket({
      construct: this,
      appName,
      stage,
    });

    // 2. AWS Lambda for Backend Axum API
    const apiLambda = new resource.AwsLambda({
      construct: this,
      appName,
      stage,
      dbBucket: dbBucket.resource,
    });

    // 3. API Gateway
    new resource.ApiGateway({
      construct: this,
      appName,
      stage,
      apiLambda: apiLambda.resource,
    });

    // 4. S3 Bucket for Frontend App
    const s3WebApp = new resource.S3WebApp({
      construct: this,
      appName,
      stage,
    });

    // 5. CloudFront Distribution
    new resource.CloudfrontDist({
      construct: this,
      appName,
      stage,
      webAppBucket: s3WebApp.resource,
    });
  }
}
