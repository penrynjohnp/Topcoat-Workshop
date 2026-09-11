#!/usr/bin/env sh
# azd postprovision hook: read-only control-plane verification. By the time
# this runs, infra/modules/secret-bootstrap.bicep has already guaranteed the
# cookie signing key exists in Key Vault (via its own bootstrap identity,
# ahead of the app module — ACA fails the deployment outright if a
# `keyVaultUrl` secret reference points at a missing secret, so that ordering
# problem is fixed in Bicep, not here).
#
# This script never reads secret values and never mutates infrastructure. It
# only confirms, via `az containerapp show`, that the deployed Container App
# is actually using its own user-assigned identity and that its Key Vault
# secret reference for SLIPWAY_COOKIE_KEY resolves through that same identity
# to the expected vault + secret name.
set -eu

command -v az >/dev/null || { echo "error: az CLI is required" >&2; exit 1; }

# Fail fast and clearly if `azd provision` hasn't produced the outputs this
# check depends on, rather than letting a blank value silently pass.
required_outputs="AZURE_RESOURCE_GROUP AZURE_CONTAINER_APP_NAME AZURE_KEY_VAULT_ENDPOINT AZURE_KEY_VAULT_SECRET_NAME AZURE_USER_ASSIGNED_IDENTITY_ID"
missing=0
for name in $required_outputs; do
  eval "value=\${$name:-}"
  if [ -z "$value" ]; then
    echo "error: expected azd output '$name' is not set" >&2
    missing=1
  fi
done
[ "$missing" -eq 0 ] || exit 1

# The Container App's own user-assigned identity must be attached — this is
# what it uses for both ACR pull and the Key Vault secret reference.
attached_identities=$(az containerapp show \
  --resource-group "$AZURE_RESOURCE_GROUP" \
  --name "$AZURE_CONTAINER_APP_NAME" \
  --query 'keys(identity.userAssignedIdentities)' \
  --output tsv)
expected_identity_lower=$(printf '%s' "$AZURE_USER_ASSIGNED_IDENTITY_ID" | tr '[:upper:]' '[:lower:]')
identity_match=0
for identity in $attached_identities; do
  identity_lower=$(printf '%s' "$identity" | tr '[:upper:]' '[:lower:]')
  if [ "$identity_lower" = "$expected_identity_lower" ]; then
    identity_match=1
    break
  fi
done
if [ "$identity_match" -ne 1 ]; then
  echo "error: container app '$AZURE_CONTAINER_APP_NAME' is not using the expected user-assigned identity ($AZURE_USER_ASSIGNED_IDENTITY_ID)" >&2
  exit 1
fi

# The secret reference must point at the vault/secret secret-bootstrap.bicep
# created, and must resolve through the app's own identity — never metadata
# about the secret's value.
expected_kv_url="${AZURE_KEY_VAULT_ENDPOINT}secrets/${AZURE_KEY_VAULT_SECRET_NAME}"
secret_identity=$(az containerapp show \
  --resource-group "$AZURE_RESOURCE_GROUP" \
  --name "$AZURE_CONTAINER_APP_NAME" \
  --query "properties.configuration.secrets[?keyVaultUrl=='$expected_kv_url'].identity | [0]" \
  --output tsv)

if [ -z "$secret_identity" ]; then
  echo "error: container app '$AZURE_CONTAINER_APP_NAME' has no secret referencing $expected_kv_url" >&2
  exit 1
fi

secret_identity_lower=$(printf '%s' "$secret_identity" | tr '[:upper:]' '[:lower:]')
if [ "$secret_identity_lower" != "$expected_identity_lower" ]; then
  echo "error: Key Vault secret reference on '$AZURE_CONTAINER_APP_NAME' does not resolve through the app's user-assigned identity (found: $secret_identity)" >&2
  exit 1
fi

echo "Verified: container app '$AZURE_CONTAINER_APP_NAME' uses its own user-assigned identity and its Key Vault secret reference resolves through that identity."
