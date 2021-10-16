data "aws_region" "current" {}
data "aws_caller_identity" "current" {}

locals {
  app = "test_multi_exec"
}

provider "aws" {
  region = "eu-central-1"
}
