resource "aws_cloudwatch_log_group" "lambda_log_group" {
  name              = "/aws/lambda/${local.app}"
  retention_in_days = 7
}

resource "aws_lambda_function" "lambda" {
  filename         = "${path.module}/${local.app}.zip"
  function_name    = local.app
  role             = aws_iam_role.iam_for_lambda.arn
  handler          = "unrelevant"
  runtime          = "provided.al2"
  timeout          = 60
  source_code_hash = filebase64sha256("${path.module}/${local.app}.zip")

  vpc_config {
    security_group_ids = [aws_security_group.lambda_rotation_secgroup.id]
    subnet_ids         = module.vpc.private_subnets
  }
}

resource "aws_security_group" "lambda_rotation_secgroup" {
  name_prefix = local.app
  description = "SecGroup for Lambda"
  vpc_id      = module.vpc.vpc_id

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_iam_policy" "lambda_to_secretmanager_policy" {
  name_prefix = "${local.app}-lambda-to-secretmanager-policy"
  path        = "/"

  policy = jsonencode({
    "Version" : "2012-10-17",
    "Statement" : [
      {
        "Condition" : {
          "StringEquals" : {
            "secretsmanager:resource/AllowRotationLambdaArn" : "arn:aws:lambda:${data.aws_region.current.name}:${data.aws_caller_identity.current.account_id}:function:${aws_lambda_function.lambda.function_name}"
          }
        },
        "Action" : [
          "secretsmanager:DescribeSecret",
          "secretsmanager:GetSecretValue",
          "secretsmanager:PutSecretValue",
          "secretsmanager:UpdateSecretVersionStage"
        ],
        "Resource" : "arn:aws:secretsmanager:${data.aws_region.current.name}:${data.aws_caller_identity.current.account_id}:secret:*",
        "Effect" : "Allow"
      },
      {
        "Action" : [
          "secretsmanager:GetRandomPassword"
        ],
        "Resource" : "*",
        "Effect" : "Allow"
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "lambda_role_policy_attachment3" {
  role       = aws_iam_role.iam_for_lambda.name
  policy_arn = aws_iam_policy.lambda_to_secretmanager_policy.arn
}

