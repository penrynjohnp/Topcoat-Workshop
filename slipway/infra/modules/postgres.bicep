// PostgreSQL Flexible Server with Microsoft Entra-only authentication: password
// auth is disabled entirely, and the workshop's user-assigned managed identity
// is registered as the Entra administrator so `production::connect_managed_postgres`
// can authenticate with a token instead of a password.
metadata description = 'Creates the PostgreSQL Flexible Server used by Slipway.'

param name string
param location string
param tags object = {}
param databaseName string = 'slipway'

@description('Object ID of the user-assigned managed identity registered as the Entra administrator.')
param administratorPrincipalId string

@description('Display name of the user-assigned managed identity (must match its Entra service principal name).')
param administratorPrincipalName string

resource server 'Microsoft.DBforPostgreSQL/flexibleServers@2024-08-01' = {
  name: name
  location: location
  tags: tags
  sku: {
    name: 'Standard_B1ms'
    tier: 'Burstable'
  }
  properties: {
    version: '16'
    storage: { storageSizeGB: 32 }
    backup: {
      backupRetentionDays: 7
      geoRedundantBackup: 'Disabled'
    }
    highAvailability: { mode: 'Disabled' }
    network: { publicNetworkAccess: 'Enabled' }
    authConfig: {
      activeDirectoryAuth: 'Enabled'
      passwordAuth: 'Disabled'
      tenantId: subscription().tenantId
    }
  }
}

resource administrator 'Microsoft.DBforPostgreSQL/flexibleServers/administrators@2024-08-01' = {
  parent: server
  name: administratorPrincipalId
  properties: {
    principalType: 'ServicePrincipal'
    principalName: administratorPrincipalName
    tenantId: subscription().tenantId
  }
}

resource database 'Microsoft.DBforPostgreSQL/flexibleServers/databases@2024-08-01' = {
  parent: server
  name: databaseName
  properties: {
    charset: 'UTF8'
    collation: 'en_US.utf8'
  }
}

// Allows Container Apps' dynamic Azure egress to reach the public server.
// The 0.0.0.0 rule admits connections from Azure services in any tenant, so
// Entra authentication and TLS remain essential. Use VNet integration and a
// private endpoint for a production deployment.
resource allowAzureServices 'Microsoft.DBforPostgreSQL/flexibleServers/firewallRules@2024-08-01' = {
  parent: server
  name: 'AllowAllAzureServicesAndResourcesWithinAzureIps'
  properties: {
    startIpAddress: '0.0.0.0'
    endIpAddress: '0.0.0.0'
  }
}

output host string = server.properties.fullyQualifiedDomainName
output databaseName string = database.name
output serverName string = server.name
