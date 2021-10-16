#!/usr/bin/env bash
set -e

terraform apply -auto-approve

echo "Executing Test. Please wait"
RESPONSE=$(( aws lambda invoke \
    --function-name "test_timeout"  \
    --invocation-type RequestResponse \
    --cli-binary-format raw-in-base64-out \
    --payload '{"timeout_secs":60}' /dev/stderr > /dev/null ) 2>&1 )
MESSAGE=$(echo "${RESPONSE}" | jq -r '.errorMessage')
if [ "${MESSAGE}" = "Lambda failed by running into a timeout" ]; then
    echo "Test successfull"
else
    echo "Test failed. Reponse from lambda is: ${RESPONSE}"
fi

echo "Press Enter to cleanup test"
read
terraform destroy -auto-approve
