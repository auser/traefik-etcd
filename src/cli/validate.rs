use crate::core::Validate;
use crate::{core::client::StoreClient, error::TraefikResult, features::etcd::Etcd, TraefikConfig};

pub async fn run(
    _client: &StoreClient<Etcd>,
    traefik_config: &mut TraefikConfig,
) -> TraefikResult<()> {
    println!("Validating Traefik configuration...");
    traefik_config.validate()?;
    println!("Validation completed successfully");

    Ok(())
}
