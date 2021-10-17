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
    --payload 'null' /dev/stderr > /dev/null ) 2>&1 )
if [ "${RESPONSE}" -eq "1" ]; then
    RESPONSE=$(( aws lambda invoke \
        --function-name "${LAMBDA_NAME}"  \
        --invocation-type RequestResponse \
        --cli-binary-format raw-in-base64-out \
        --payload 'null' /dev/stderr > /dev/null ) 2>&1 )
    if [ "${RESPONSE}" -eq "2" ]; then
        echo "Test successfull"
    else
        echo "Test failed. Second reponse from lambda is: ${RESPONSE}"
        EXIT=1
    fi
else
    echo "Test failed. First reponse from lambda is: ${RESPONSE}"
    EXIT=2
fi

echo "Destroying resources. Please wait"
terraform destroy -auto-approve > ./output
exit "${EXIT}"
