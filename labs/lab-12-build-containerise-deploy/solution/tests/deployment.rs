const DOCKERFILE: &str = include_str!("../Dockerfile");
const DOCKERIGNORE: &str = include_str!("../../../../.dockerignore");
const AZURE_YAML: &str = include_str!("../azure.yaml");
const MAIN_BICEP: &str = include_str!("../infra/main.bicep");
const APP_BICEP: &str = include_str!("../infra/modules/app.bicep");
const REGISTRY_BICEP: &str = include_str!("../infra/modules/registry.bicep");
const POSTGRES_BICEP: &str = include_str!("../infra/modules/postgres.bicep");
const KEY_VAULT_BICEP: &str = include_str!("../infra/modules/keyvault.bicep");
const SECRET_BOOTSTRAP_BICEP: &str = include_str!("../infra/modules/secret-bootstrap.bicep");
const HOOK: &str = include_str!("../scripts/azd-postprovision.sh");
const WORKFLOW: &str = include_str!("../../../../.github/workflows/azure-dev.yml");

#[test]
fn image_keeps_the_release_binary_and_asset_catalog_together() {
    assert!(DOCKERFILE.contains("cargo build --release -p slipway"));
    assert!(DOCKERFILE.contains("topcoat asset bundle --release -p slipway"));
    assert!(DOCKERFILE.contains("target/release/slipway"));
    assert!(DOCKERFILE.contains("target/release/assets"));
    assert!(DOCKERFILE.contains("USER nonroot"));
    assert!(DOCKERFILE.contains("HOST=0.0.0.0"));
    assert!(DOCKERFILE.contains("PORT=8080"));
    assert!(
        DOCKERIGNORE
            .lines()
            .any(|line| line == "rust-toolchain.toml")
    );
}

#[test]
fn azd_builds_from_the_workspace_and_runs_the_postprovision_hook() {
    assert!(AZURE_YAML.contains("project: ../../.."));
    assert!(AZURE_YAML.contains("language: docker"));
    assert!(AZURE_YAML.contains("host: containerapp"));
    assert!(AZURE_YAML.contains("scripts/azd-postprovision.sh"));
    assert!(MAIN_BICEP.contains("output AZURE_CONTAINER_REGISTRY_ENDPOINT"));
    assert!(MAIN_BICEP.contains("output WEB_URI"));
}

#[test]
fn azure_resources_use_identity_instead_of_passwords() {
    assert!(REGISTRY_BICEP.contains("adminUserEnabled: false"));
    assert!(REGISTRY_BICEP.contains("acrPullRoleId"));
    assert!(REGISTRY_BICEP.contains("pullPrincipalId"));
    assert!(POSTGRES_BICEP.contains("activeDirectoryAuth: 'Enabled'"));
    assert!(POSTGRES_BICEP.contains("passwordAuth: 'Disabled'"));
    assert!(POSTGRES_BICEP.contains("principalType: 'ServicePrincipal'"));
    assert!(KEY_VAULT_BICEP.contains("enableRbacAuthorization: true"));
    assert!(KEY_VAULT_BICEP.contains("secretsUserRoleId"));
    assert!(KEY_VAULT_BICEP.contains("bootstrapSecretsOfficer"));
    assert!(!KEY_VAULT_BICEP.contains("deployerPrincipal"));
    assert!(!APP_BICEP.contains("password"));
    assert!(!APP_BICEP.contains("username"));
}

#[test]
fn container_app_has_ingress_probes_scaling_and_key_vault_reference() {
    assert!(APP_BICEP.contains("'azd-service-name': 'web'"));
    assert!(APP_BICEP.contains("targetPort: 8080"));
    assert_eq!(APP_BICEP.matches("path: '/api/health'").count(), 2);
    assert!(APP_BICEP.contains("minReplicas: 0"));
    assert!(APP_BICEP.contains("maxReplicas: 3"));
    assert!(APP_BICEP.contains("keyVaultUrl:"));
    assert!(APP_BICEP.contains("identity: userAssignedIdentityId"));
    assert!(APP_BICEP.contains("SLIPWAY_ENV"));
    assert!(APP_BICEP.contains("APPLICATIONINSIGHTS_CONNECTION_STRING"));
}

#[test]
fn cookie_secret_is_bootstrapped_before_the_app_and_never_printed() {
    assert!(SECRET_BOOTSTRAP_BICEP.contains("Microsoft.Resources/deploymentScripts"));
    assert!(SECRET_BOOTSTRAP_BICEP.contains("head -c 64 /dev/urandom"));
    assert!(SECRET_BOOTSTRAP_BICEP.contains("az keyvault secret set"));
    assert!(SECRET_BOOTSTRAP_BICEP.contains("forceUpdateTag"));
    assert!(!SECRET_BOOTSTRAP_BICEP.contains("echo \"$key\""));
    assert!(MAIN_BICEP.contains("dependsOn: [\n    secretBootstrap"));
}

#[test]
fn postprovision_hook_only_verifies_control_plane_wiring() {
    assert!(HOOK.contains("az containerapp show"));
    assert!(HOOK.contains("AZURE_USER_ASSIGNED_IDENTITY_ID"));
    assert!(HOOK.contains("AZURE_KEY_VAULT_SECRET_NAME"));
    assert!(!HOOK.contains("az keyvault secret show"));
    assert!(!HOOK.contains("az keyvault secret set"));
}

#[test]
fn deployment_workflow_uses_github_oidc_without_a_client_secret() {
    assert!(WORKFLOW.contains("id-token: write"));
    assert!(WORKFLOW.contains("federated-credential-provider \"github\""));
    assert!(WORKFLOW.contains("azd provision --no-prompt"));
    assert!(WORKFLOW.contains("azd deploy --no-prompt"));
    assert!(!WORKFLOW.contains("AZURE_CLIENT_SECRET"));
    assert!(!WORKFLOW.contains("secrets.AZURE"));
}
