#!/usr/bin/env bash
set -e

terraform apply -auto-approve

echo "Executing Test. Please wait"
PW=$(cat "./password")

SECRET_NAME=$(aws secretsmanager list-secrets | \
    jq -r '.SecretList[] | .Name' | \
    grep "est_postgres_rotation" | \
    head -n 1)
FOUND=false
for i in $(seq 1 60); do 
    PW_NEW=$(aws secretsmanager get-secret-value --secret-id "${SECRET_NAME}" | \
            jq -r '.SecretString' | \
            jq -r '.password')
    if [ "${PW_NEW}" = "${PW}" ]; then
        sleep 1
    else
        FOUND=true
        break
    fi
done
if [ "${FOUND}" = true ] ; then
    echo "Test successfull"
else 
    echo "Test failed. Password is still: ${PW_NEW}"
fi

echo "Press Enter to cleanup test"
cat <<EOF
Hint: Lambda creates Network Interfaces which are not cleaned up by terraform.
It takes about 20 minutes for them to not be in use anymore, afterwards they
must be deleted manually.
EOF

read
terraform destroy -auto-approve
