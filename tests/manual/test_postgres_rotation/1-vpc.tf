module "vpc" {
  source = "terraform-aws-modules/vpc/aws"

  name = local.app
  cidr = "10.0.0.0/16"

  azs             = ["eu-central-1a", "eu-central-1b", "eu-central-1c"]
  private_subnets = ["10.0.0.0/20", "10.0.16.0/20", "10.0.32.0/20"]
  public_subnets  = ["10.0.48.0/20", "10.0.64.0/20", "10.0.80.0/20"]

  enable_nat_gateway      = true
  single_nat_gateway      = true
  map_public_ip_on_launch = true

  tags = {
    Name         = local.app
    Provisioning = "Terraform"
  }
}
