use crate::{config::TraefikConfig, error::TraefikResult, etcd::Etcd};

pub async fn run(_etcd_client: &Etcd, traefik_config: &mut TraefikConfig) -> TraefikResult<()> {
    println!("Validating Traefik configuration...");
    traefik_config.validate()?;
    println!("Validation completed successfully");

    Ok(())
}
