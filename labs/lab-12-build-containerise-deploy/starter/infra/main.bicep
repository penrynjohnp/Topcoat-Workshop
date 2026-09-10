targetScope = 'subscription'

param environmentName string
param location string

var tags = { 'azd-env-name': environmentName }

resource rg 'Microsoft.Resources/resourceGroups@2024-11-01' = {
  name: 'rg-${environmentName}'
  location: location
  tags: tags
}

// TODO(lab-12): Add monitoring, registry, app/bootstrap identities, Key Vault,
// the pre-app cookie-key deployment script, PostgreSQL, the Container Apps
// environment, and the app modules.

output AZURE_RESOURCE_GROUP string = rg.name
