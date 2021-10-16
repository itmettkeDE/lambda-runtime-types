data "aws_region" "current" {}
data "aws_caller_identity" "current" {}

locals {
  app = "test_timeout"
}

provider "aws" {
  region = "eu-central-1"
}
