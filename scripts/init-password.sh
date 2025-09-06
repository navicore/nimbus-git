#!/bin/bash
# Initialize Nimbus admin password
# This script sets up the initial admin password for Nimbus

set -e

NAMESPACE=${NIMBUS_NAMESPACE:-nimbus}
PASSWORD=${1:-}

if [ -z "$PASSWORD" ]; then
    echo "Usage: $0 <password>"
    echo "Sets the admin password for Nimbus"
    exit 1
fi

echo "Setting admin password in namespace: $NAMESPACE"

# Hash the password using Argon2
# We'll use a simple Python script since argon2 CLI might not be available
HASH=$(python3 -c "
import hashlib
import base64
import os
from argon2 import PasswordHasher

ph = PasswordHasher()
hash = ph.hash('$PASSWORD')
print(hash)
" 2>/dev/null || echo "")

if [ -z "$HASH" ]; then
    echo "Error: Could not hash password. Install argon2-cffi: pip install argon2-cffi"
    echo "Or set password directly in Kubernetes secret"
    exit 1
fi

# Encode for Kubernetes secret
USERNAME_B64=$(echo -n "admin" | base64)
HASH_B64=$(echo -n "$HASH" | base64)

# Create or update the secret
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: Secret
metadata:
  name: nimbus-owner
  namespace: $NAMESPACE
type: Opaque
data:
  username: $USERNAME_B64
  password_hash: $HASH_B64
EOF

echo "Password updated successfully!"
echo "You can now login with username: admin"