// Idempotently ensures the Slipway cookie signing key exists in Key Vault
// *before* the container app references it. Azure Container Apps validates
// `keyVaultUrl` secret references at deploy time and fails the whole
// deployment if the target secret doesn't exist yet, so this must run and
// complete ahead of the app module — not after it, as a postprovision hook
// would. It runs under its own bootstrap identity (Key Vault Secrets Officer
// only); the app's identity never gets write access to the vault.
metadata description = 'Bootstraps the cookie signing key secret with an AzureCLI deployment script.'

param location string
param tags object = {}
param keyVaultName string
param secretName string = 'slipway-cookie-key'
param userAssignedIdentityId string

@description('Changes on every deployment so the script always re-runs — the script itself is idempotent (create-if-absent), so this only matters when the secret was deleted out-of-band and needs recreating on the next `azd provision`.')
param forceUpdateTag string = utcNow()

resource bootstrap 'Microsoft.Resources/deploymentScripts@2023-08-01' = {
  name: 'cookie-key-bootstrap'
  location: location
  tags: tags
  kind: 'AzureCLI'
  identity: {
    type: 'UserAssigned'
    userAssignedIdentities: {
      '${userAssignedIdentityId}': {}
    }
  }
  properties: {
    azCliVersion: '2.65.0'
    retentionInterval: 'P1D'
    timeout: 'PT10M'
    cleanupPreference: 'OnSuccess'
    forceUpdateTag: forceUpdateTag
    environmentVariables: [
      { name: 'VAULT_NAME', value: keyVaultName }
      { name: 'SECRET_NAME', value: secretName }
    ]
    scriptContent: '''
      set -euo pipefail

      # RBAC role assignments can take a few seconds to become effective;
      # retry the permission check rather than failing the whole deployment
      # on that race.
      ready=0
      for _ in 1 2 3 4 5 6; do
        if az keyvault secret list --vault-name "$VAULT_NAME" -o none 2>/dev/null; then
          ready=1
          break
        fi
        sleep 15
      done
      if [ "$ready" -ne 1 ]; then
        echo "error: could not list secrets in $VAULT_NAME (RBAC not yet effective)" >&2
        exit 1
      fi

      if az keyvault secret show --vault-name "$VAULT_NAME" --name "$SECRET_NAME" -o none 2>/dev/null; then
        echo "cookie signing key already present in $VAULT_NAME"
      else
        # 64 random bytes, base64-encoded — topcoat::cookie::Key requires at
        # least 64 decoded bytes. Never echoed; only status lines are logged.
        key=$(head -c 64 /dev/urandom | base64 | tr -d '\n')
        az keyvault secret set --vault-name "$VAULT_NAME" --name "$SECRET_NAME" --value "$key" --output none
        unset key
        echo "cookie signing key created in $VAULT_NAME"
      fi
    '''
  }
}
