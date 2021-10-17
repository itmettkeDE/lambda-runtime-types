#!/usr/bin/env bash
set -e
EXIT=0

echo "Creating resources. Please wait"
terraform apply -auto-approve > ./output

LAMBDA_NAME="$(terraform output --raw lambda_name)"
echo "Executing Test. Please wait"
RESPONSE=$(( aws lambda invoke \
    --function-name "${LAMBDA_NAME}"  \
    --invocation-type RequestResponse \
    --cli-binary-format raw-in-base64-out \
    --payload '{"timeout_secs":60}' /dev/stderr > /dev/null ) 2>&1 )
MESSAGE=$(echo "${RESPONSE}" | jq -r '.errorMessage')
if [ "${MESSAGE}" = "Lambda failed by running into a timeout" ]; then
    echo "Test successfull"
else
    EXIT=1
    echo "Test failed. Reponse from lambda is: ${RESPONSE}"
fi

echo "Destroying resources. Please wait"
terraform destroy -auto-approve > ./output
exit "${EXIT}"
