// RBAC-authorized Key Vault holding the Toasty/Topcoat cookie signing key.
// No access policies: every access path is a role assignment.
metadata description = 'Creates the Key Vault used to store the session cookie signing key.'

param name string
param location string
param tags object = {}

@description('Principal ID of the bootstrap managed identity used only by the cookie-key deployment script. It alone can create or update secrets.')
param bootstrapPrincipalId string

@description('Principal ID of the container app user-assigned managed identity, granted read-only secret access.')
param readerPrincipalId string

var secretsOfficerRoleId = '00482a5a-887f-4fb3-b428-0adb0e78b0d5'
var secretsUserRoleId = '4633458b-17de-408a-b874-0445c86b69e6'

resource vault 'Microsoft.KeyVault/vaults@2023-07-01' = {
  name: name
  location: location
  tags: tags
  properties: {
    sku: { family: 'A', name: 'standard' }
    tenantId: subscription().tenantId
    enableRbacAuthorization: true
    enableSoftDelete: true
    softDeleteRetentionInDays: 7
    // Keep purge protection off in the disposable workshop environment so
    // `azd down --purge` can release the generated vault name.
    enablePurgeProtection: false
    publicNetworkAccess: 'Enabled'
    accessPolicies: []
  }
}

// Only the bootstrap identity — used exclusively by the cookie-key
// deployment script — can create or update secrets in this vault. Neither
// the app's own identity nor the azd deployer ever get write access.
resource bootstrapSecretsOfficer 'Microsoft.Authorization/roleAssignments@2022-04-01' = {
  name: guid(vault.id, bootstrapPrincipalId, secretsOfficerRoleId)
  scope: vault
  properties: {
    principalId: bootstrapPrincipalId
    principalType: 'ServicePrincipal'
    roleDefinitionId: subscriptionResourceId('Microsoft.Authorization/roleDefinitions', secretsOfficerRoleId)
  }
}

// The running container app resolves its `SLIPWAY_COOKIE_KEY` secret reference
// with this identity, so it only ever needs read access.
resource readerSecretsUser 'Microsoft.Authorization/roleAssignments@2022-04-01' = {
  name: guid(vault.id, readerPrincipalId, secretsUserRoleId)
  scope: vault
  properties: {
    principalId: readerPrincipalId
    principalType: 'ServicePrincipal'
    roleDefinitionId: subscriptionResourceId('Microsoft.Authorization/roleDefinitions', secretsUserRoleId)
  }
}

output id string = vault.id
output name string = vault.name
output uri string = vault.properties.vaultUri
