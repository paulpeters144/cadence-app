import * as cdk from "aws-cdk-lib";
import type { Construct } from "constructs";
import * as cloudfront from "aws-cdk-lib/aws-cloudfront";
import * as origins from "aws-cdk-lib/aws-cloudfront-origins";

interface CloudfrontDistProps {
  construct: Construct;
  appName: string;
  stage: string;
  webAppBucket: cdk.aws_s3.Bucket;
}

export class CloudfrontDist {
  resource: cloudfront.Distribution;

  constructor(props: CloudfrontDistProps) {
    const { construct, appName, stage, webAppBucket } = props;
    const distName = `${appName}-${stage}-cloudfront-dist`;

    const origin = origins.S3BucketOrigin.withOriginAccessControl(webAppBucket);

    this.resource = new cloudfront.Distribution(construct, distName, {
      defaultBehavior: {
        origin: origin,
        viewerProtocolPolicy: cloudfront.ViewerProtocolPolicy.REDIRECT_TO_HTTPS,
        compress: true,
        cachePolicy: cloudfront.CachePolicy.CACHING_DISABLED, // Change if needed
      },
      defaultRootObject: "index.html",
      errorResponses: [
        {
          httpStatus: 404,
          responseHttpStatus: 200,
          responsePagePath: "/index.html",
        },
      ],
      priceClass: cloudfront.PriceClass.PRICE_CLASS_100,
      enabled: true,
      comment: `CloudFront distribution for ${appName} ${stage}`,
    });

    new cdk.CfnOutput(construct, "CloudfrontDistDomainName", {
      value: this.resource.domainName,
      description: "Cloudfront Dist Domain Name",
    });
  }
}
