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
  throw new Error(
    "stage context or env is not set. Use -c stage=uat or -c stage=production",
  );
}

const appName = "cadence-app";
const stackName = `${appName}-${context.stage}`;

const account = process.env.AWS_ACCOUNT;
const region = process.env.AWS_REGION;

if (!account || !region) {
  throw new Error(
    "Missing AWS environment variables. " +
      "Please ensure AWS_ACCOUNT and AWS_REGION are set in your .env file or environment.",
  );
}

new InfraStack(app, stackName, {
  env: {
    account,
    region,
  },
  stackName: stackName,
  tags: { stage: context.stage, appName },
});
