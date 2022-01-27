data "aws_region" "current" {}
data "aws_caller_identity" "current" {}

locals {
  app     = "test_multi_exec"
  iam_arn = "arn:aws:iam::118325176989:role/lambda-role-default-ci"
}

provider "aws" {
  region = "eu-central-1"

  default_tags {
    tags = {
      Name         = local.app
      Provisioning = "Terraform"
    }
  }
}
