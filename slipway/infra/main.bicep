// Subscription-scoped entry point (azd convention): creates the resource
// group and wires the monitoring, registry, identity, Key Vault, PostgreSQL,
// Container Apps environment, and app modules together.
targetScope = 'subscription'

@minLength(1)
@maxLength(64)
@description('Name of the azd environment; used to derive a unique resource token.')
param environmentName string

@minLength(1)
@description('Azure region for all resources.')
param location string

@description('Full container image reference injected by azd as SERVICE_WEB_IMAGE_NAME after `azd deploy` has pushed one. Left empty for the first `azd provision`.')
param webImageName string = ''

@description('`RUST_LOG` value for the container app.')
param rustLogLevel string = 'info'

var abbrs = loadJsonContent('./abbreviations.json')
var resourceToken = toLower(uniqueString(subscription().id, environmentName, location))
var tags = {
  'azd-env-name': environmentName
}
var cookieKeySecretName = 'slipway-cookie-key'

resource rg 'Microsoft.Resources/resourceGroups@2024-11-01' = {
  name: '${abbrs.resourcesResourceGroups}${environmentName}'
  location: location
  tags: tags
}

module monitoring './modules/monitoring.bicep' = {
  name: 'monitoring'
  scope: rg
  params: {
    logAnalyticsName: '${abbrs.operationalInsightsWorkspaces}${resourceToken}'
    applicationInsightsName: '${abbrs.insightsComponents}${resourceToken}'
    location: location
    tags: tags
  }
}

module registry './modules/registry.bicep' = {
  name: 'registry'
  scope: rg
  params: {
    name: '${abbrs.containerRegistryRegistries}${resourceToken}'
    location: location
    tags: tags
    pullPrincipalId: identity.outputs.principalId
  }
}

module identity './modules/identity.bicep' = {
  name: 'identity'
  scope: rg
  params: {
    name: '${abbrs.managedIdentityUserAssignedIdentities}app-${resourceToken}'
    location: location
    tags: tags
  }
}

// A separate, narrowly-scoped identity for the cookie-key deployment script.
// It only ever holds Key Vault Secrets Officer — the app's own identity
// (above) never gets write access to the vault.
module bootstrapIdentity './modules/identity.bicep' = {
  name: 'bootstrap-identity'
  scope: rg
  params: {
    name: '${abbrs.managedIdentityUserAssignedIdentities}bootstrap-${resourceToken}'
    location: location
    tags: tags
  }
}

module keyVault './modules/keyvault.bicep' = {
  name: 'keyvault'
  scope: rg
  params: {
    name: '${abbrs.keyVaultVaults}${resourceToken}'
    location: location
    tags: tags
    bootstrapPrincipalId: bootstrapIdentity.outputs.principalId
    readerPrincipalId: identity.outputs.principalId
  }
}

// Runs before the app module and creates the cookie signing key secret if
// it's absent — ACA fails the whole deployment if a `keyVaultUrl` secret
// reference points at a secret that doesn't exist yet, so this can't run as
// a postprovision hook after the app is created.
module secretBootstrap './modules/secret-bootstrap.bicep' = {
  name: 'secret-bootstrap'
  scope: rg
  params: {
    location: location
    tags: tags
    keyVaultName: keyVault.outputs.name
    secretName: cookieKeySecretName
    userAssignedIdentityId: bootstrapIdentity.outputs.id
  }
}

module postgres './modules/postgres.bicep' = {
  name: 'postgres'
  scope: rg
  params: {
    name: '${abbrs.dBforPostgreSQLFlexibleServers}${resourceToken}'
    location: location
    tags: tags
    administratorPrincipalId: identity.outputs.principalId
    administratorPrincipalName: identity.outputs.name
  }
}

module containerAppsEnvironment './modules/environment.bicep' = {
  name: 'container-apps-environment'
  scope: rg
  params: {
    name: '${abbrs.appManagedEnvironments}${resourceToken}'
    location: location
    tags: tags
    logAnalyticsWorkspaceName: monitoring.outputs.logAnalyticsWorkspaceName
  }
}

module app './modules/app.bicep' = {
  name: 'app'
  scope: rg
  params: {
    name: '${abbrs.appContainerApps}web-${resourceToken}'
    location: location
    tags: tags
    environmentId: containerAppsEnvironment.outputs.id
    acrLoginServer: registry.outputs.loginServer
    userAssignedIdentityId: identity.outputs.id
    userAssignedIdentityClientId: identity.outputs.clientId
    keyVaultUri: keyVault.outputs.uri
    keyVaultSecretName: cookieKeySecretName
    postgresHost: postgres.outputs.host
    postgresDatabaseName: postgres.outputs.databaseName
    postgresAdministratorName: identity.outputs.name
    applicationInsightsConnectionString: monitoring.outputs.applicationInsightsConnectionString
    imageName: webImageName
    rustLogLevel: rustLogLevel
  }
  // The Key Vault secret reference below must resolve to an existing secret
  // at deploy time, so the bootstrap script has to finish first.
  dependsOn: [
    secretBootstrap
  ]
}

// Outputs are written to `.azure/<env>/.env` and surfaced by `azd env get-values`.
// Nothing secret is emitted; the cookie key lives only in Key Vault.
output AZURE_LOCATION string = location
output AZURE_TENANT_ID string = tenant().tenantId
output AZURE_RESOURCE_GROUP string = rg.name

// Well-known names azd's `containerapp`/docker service target reads to build
// and push the image.
output AZURE_CONTAINER_REGISTRY_ENDPOINT string = registry.outputs.loginServer
output AZURE_CONTAINER_REGISTRY_NAME string = registry.outputs.name

// Consumed by scripts/azd-postprovision.sh, which verifies (read-only,
// control-plane only) that the app is wired to these — it never creates or
// reads the secret's value; secret-bootstrap.bicep already guaranteed it
// exists before the app module ran.
output AZURE_KEY_VAULT_NAME string = keyVault.outputs.name
output AZURE_KEY_VAULT_ENDPOINT string = keyVault.outputs.uri
output AZURE_KEY_VAULT_SECRET_NAME string = cookieKeySecretName

output AZURE_CONTAINER_APP_NAME string = app.outputs.name
output AZURE_USER_ASSIGNED_IDENTITY_ID string = identity.outputs.id
output AZURE_USER_ASSIGNED_IDENTITY_NAME string = identity.outputs.name
output AZURE_USER_ASSIGNED_IDENTITY_CLIENT_ID string = identity.outputs.clientId

output SLIPWAY_DATABASE_HOST string = postgres.outputs.host
output SLIPWAY_DATABASE_NAME string = postgres.outputs.databaseName
output SLIPWAY_DATABASE_USER string = identity.outputs.name

// Public URL, printed by `azd up`/`azd show` and used for the post-deploy
// smoke check (`curl .../api/health`).
output SERVICE_WEB_ENDPOINT_URL string = app.outputs.uri
output WEB_URI string = app.outputs.uri
