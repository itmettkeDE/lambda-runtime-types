resource "aws_secretsmanager_secret" "secret" {
  name_prefix = local.app
}

resource "aws_secretsmanager_secret_version" "secret_inital" {
  secret_id = aws_secretsmanager_secret.secret.id
  secret_string = jsonencode({
    host     = module.rds.rds_cluster_endpoint
    port     = module.rds.rds_cluster_port
    database = module.rds.rds_cluster_database_name
    user     = module.rds.rds_cluster_master_username
    password = module.rds.rds_cluster_master_password
  })
}

resource "aws_lambda_permission" "allow_secret_manager_call_Lambda" {
  function_name = aws_lambda_function.lambda.arn
  statement_id  = "AllowExecutionSecretManager"
  action        = "lambda:InvokeFunction"
  principal     = "secretsmanager.amazonaws.com"
}

resource "aws_secretsmanager_secret_rotation" "tnbl_password_rotation" {
  depends_on = [
    module.rds,
    aws_iam_role_policy_attachment.lambda_role_policy_attachment3,
    aws_secretsmanager_secret_version.secret_inital,
  ]
  secret_id           = aws_secretsmanager_secret.secret.id
  rotation_lambda_arn = aws_lambda_function.lambda.arn

  rotation_rules {
    automatically_after_days = 30
  }
}

resource "local_file" "password_file" {
  content  = module.rds.rds_cluster_master_password
  filename = "${path.module}/password"
}
