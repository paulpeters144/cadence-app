import * as cdk from "aws-cdk-lib";
import type { Construct } from "constructs";

interface AwsLambdaProps {
  construct: Construct;
  appName: string;
  stage: string;
}

export class AwsLambda {
  resource: cdk.aws_lambda.Function;

  constructor(props: AwsLambdaProps) {
    const { construct, appName, stage } = props;
    const apiLambdaName = `${appName}-lambda-${stage}`;

    this.resource = new cdk.aws_lambda.Function(construct, apiLambdaName, {
      functionName: apiLambdaName,
      timeout: cdk.Duration.seconds(8),
      runtime: cdk.aws_lambda.Runtime.PROVIDED_AL2023,
      code: cdk.aws_lambda.Code.fromAsset("../backend/dist"),
      handler: "bootstrap", // Standard for Rust
      architecture: cdk.aws_lambda.Architecture.ARM_64,
      memorySize: 512,
    });
  }
}
