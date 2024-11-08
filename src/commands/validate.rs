use crate::{
    common::{error::TraefikResult, etcd::Etcd},
    config::TraefikConfig,
};

pub async fn run(_etcd_client: &Etcd, traefik_config: &TraefikConfig) -> TraefikResult<()> {
    println!("Validating Traefik configuration...");
    traefik_config.validate()?;
    println!("Validation completed successfully");

    Ok(())
}
