#!/usr/bin/env bash
set -e

terraform apply -auto-approve

echo "Executing Test. Please wait"
RESPONSE=$(( aws lambda invoke \
    --function-name "test_multi_exec"  \
    --invocation-type RequestResponse \
    --cli-binary-format raw-in-base64-out \
    --payload 'null' /dev/stderr > /dev/null ) 2>&1 )
if [ "${RESPONSE}" -eq "1" ]; then
    RESPONSE=$(( aws lambda invoke \
        --function-name "test_multi_exec"  \
        --invocation-type RequestResponse \
        --cli-binary-format raw-in-base64-out \
        --payload 'null' /dev/stderr > /dev/null ) 2>&1 )
    if [ "${RESPONSE}" -eq "2" ]; then
        echo "Test successfull"
    else
        echo "Test failed. Second reponse from lambda is: ${RESPONSE}"
    fi
else
    echo "Test failed. First reponse from lambda is: ${RESPONSE}"
fi

echo "Press Enter to cleanup test"
read
terraform destroy -auto-approve
