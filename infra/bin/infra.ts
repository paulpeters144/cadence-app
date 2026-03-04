import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { InfraStack } from "../lib/infra-stack";
import * as dotenv from "dotenv";

dotenv.config();

export interface StageCtx {
  stage: string;
}

const app = new cdk.App();

const stage = app.node.tryGetContext("stage") as string;
const context = app.node.tryGetContext(stage) as StageCtx;

if (!context || !context.stage) {
  throw new Error("stage context or env is not set. Use -c stage=uat or -c stage=production");
}

const appName = "cadence-app";
const stackName = `${appName}-${context.stage}`;

new InfraStack(app, stackName, {
  env: {
    account: process.env.AWS_ACCOUNT || process.env.CDK_DEFAULT_ACCOUNT,
    region: process.env.AWS_REGION || process.env.CDK_DEFAULT_REGION,
  },
  stackName: stackName,
  tags: { stage: context.stage, appName },
});
