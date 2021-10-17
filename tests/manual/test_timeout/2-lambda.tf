resource "time_static" "current" {}

resource "aws_cloudwatch_log_group" "lambda_log_group" {
  name              = "/aws/lambda/${local.app}_${time_static.current.unix}"
  retention_in_days = 7
}

resource "aws_lambda_function" "lambda" {
  filename         = "${path.module}/${local.app}.zip"
  function_name    = "${local.app}_${time_static.current.unix}"
  role             = local.iam_arn
  handler          = "unrelevant"
  runtime          = "provided.al2"
  timeout          = 2
  source_code_hash = filebase64sha256("${path.module}/${local.app}.zip")
}
