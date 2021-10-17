resource "aws_security_group" "db_sec_group" {
  name_prefix = local.app
  description = "SecGroup for Database Access"
  vpc_id      = module.vpc.vpc_id

  ingress {
    description = "Allow acces for rotation"
    from_port   = local.port
    to_port     = local.port
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

module "rds" {
  source = "terraform-aws-modules/rds-aurora/aws"
  name   = replace(local.app, "_", "-")

  engine              = "aurora-postgresql"
  engine_version      = "11.9"
  instance_type       = "db.t3.medium"
  storage_encrypted   = true
  publicly_accessible = false

  database_name = local.app
  username      = local.app
  port          = local.port

  vpc_id                 = module.vpc.vpc_id
  vpc_security_group_ids = [aws_security_group.db_sec_group.id]
  subnets                = module.vpc.private_subnets

  iam_database_authentication_enabled = true

  preferred_maintenance_window = "Sat:00:00-Sat:03:00"
  preferred_backup_window      = "03:00-06:00"

  replica_count       = 1
  monitoring_interval = 10

  copy_tags_to_snapshot = true
}
