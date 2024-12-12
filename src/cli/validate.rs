use crate::{core::client::StoreClient, error::TraefikResult, features::etcd::Etcd, TraefikConfig};

pub async fn run(
    _client: &StoreClient<Etcd>,
    traefik_config: &mut TraefikConfig,
) -> TraefikResult<()> {
    println!("Validating Traefik configuration...");
    traefik_config.validate_config()?;
    println!("Validation completed successfully");

    Ok(())
}
