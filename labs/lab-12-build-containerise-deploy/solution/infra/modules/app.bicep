// The Slipway container app: pulls from ACR via its user-assigned managed
// identity, resolves the cookie signing key from Key Vault via the same
// identity, and authenticates to PostgreSQL with an Entra token (no secrets
// for the database at all).
metadata description = 'Creates the Slipway Container App.'

param name string
param location string
param tags object = {}
param environmentId string
param acrLoginServer string
param userAssignedIdentityId string
param userAssignedIdentityClientId string
param keyVaultUri string
param keyVaultSecretName string
param postgresHost string
param postgresDatabaseName string
param postgresAdministratorName string
param applicationInsightsConnectionString string

@description('Full image reference, injected by azd as SERVICE_WEB_IMAGE_NAME once `azd deploy` has built and pushed one. Left empty on the first `azd provision`, before any image exists; the placeholder keeps that first deployment valid.')
param imageName string = ''

@description('RUST_LOG value for the container app.')
param rustLogLevel string = 'info'

var placeholderImage = 'mcr.microsoft.com/azuredocs/containerapps-helloworld:latest'
var containerImage = !empty(imageName) ? imageName : placeholderImage
var cookieKeySecretName = 'cookie-key'

resource app 'Microsoft.App/containerApps@2024-03-01' = {
  name: name
  location: location
  tags: union(tags, { 'azd-service-name': 'web' })
  identity: {
    type: 'UserAssigned'
    userAssignedIdentities: {
      '${userAssignedIdentityId}': {}
    }
  }
  properties: {
    environmentId: environmentId
    configuration: {
      activeRevisionsMode: 'Single'
      ingress: {
        external: true
        targetPort: 8080
        transport: 'auto'
        allowInsecure: false
      }
      registries: [
        {
          server: acrLoginServer
          identity: userAssignedIdentityId
        }
      ]
      secrets: [
        {
          name: cookieKeySecretName
          keyVaultUrl: '${keyVaultUri}secrets/${keyVaultSecretName}'
          identity: userAssignedIdentityId
        }
      ]
    }
    template: {
      containers: [
        {
          name: 'web'
          image: containerImage
          resources: {
            cpu: json('0.5')
            memory: '1Gi'
          }
          env: [
            { name: 'HOST', value: '0.0.0.0' }
            { name: 'PORT', value: '8080' }
            { name: 'RUST_LOG', value: rustLogLevel }
            { name: 'SLIPWAY_ENV', value: 'production' }
            { name: 'APPLICATIONINSIGHTS_CONNECTION_STRING', value: applicationInsightsConnectionString }
            { name: 'SLIPWAY_DATABASE_HOST', value: postgresHost }
            { name: 'SLIPWAY_DATABASE_NAME', value: postgresDatabaseName }
            { name: 'SLIPWAY_DATABASE_USER', value: postgresAdministratorName }
            { name: 'AZURE_CLIENT_ID', value: userAssignedIdentityClientId }
            { name: 'SLIPWAY_COOKIE_KEY', secretRef: cookieKeySecretName }
          ]
          probes: [
            {
              type: 'Liveness'
              httpGet: { path: '/api/health', port: 8080 }
              initialDelaySeconds: 30
              periodSeconds: 10
              timeoutSeconds: 5
            }
            {
              type: 'Readiness'
              httpGet: { path: '/api/health', port: 8080 }
              initialDelaySeconds: 5
              periodSeconds: 5
              timeoutSeconds: 3
            }
          ]
        }
      ]
      scale: {
        minReplicas: 0
        maxReplicas: 3
        rules: [
          {
            name: 'http'
            http: { metadata: { concurrentRequests: '20' } }
          }
        ]
      }
    }
  }
}

output name string = app.name
output fqdn string = app.properties.configuration.ingress.fqdn
output uri string = 'https://${app.properties.configuration.ingress.fqdn}'
