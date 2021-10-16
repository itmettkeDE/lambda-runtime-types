resource "aws_cloudwatch_log_group" "lambda_log_group" {
  name              = "/aws/lambda/${local.app}"
  retention_in_days = 7

  tags = {
    Name         = "${local.app}"
    Provisioning = "Terraform"
  }
}

resource "aws_lambda_function" "lambda" {
  filename         = "${path.module}/${local.app}.zip"
  function_name    = local.app
  role             = aws_iam_role.iam_for_lambda.arn
  handler          = "unrelevant"
  runtime          = "provided.al2"
  timeout          = 2
  source_code_hash = filebase64sha256("${path.module}/${local.app}.zip")

  tags = {
    Name         = local.app
    Provisioning = "Terraform"
  }
}
