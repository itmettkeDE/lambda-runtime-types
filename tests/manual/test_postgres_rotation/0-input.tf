data "aws_region" "current" {}
data "aws_caller_identity" "current" {}

locals {
  app  = "test_postgres_rotation"
  port = 5432
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
