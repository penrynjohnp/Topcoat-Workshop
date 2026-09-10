#!/usr/bin/env sh
set -eu

# TODO(lab-12): Read the non-secret azd outputs and verify that the Container
# App uses its expected identity for the expected Key Vault reference. Secret
# creation belongs in the pre-app Bicep deployment script, not this hook.
echo "Postprovision verification is not implemented yet."
