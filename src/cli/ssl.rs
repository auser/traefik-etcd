use clap::Args;

use crate::{core::client::StoreClient, error::TraefikResult, features::etcd::Etcd, TraefikConfig};

#[derive(Args, Debug)]
pub struct SslCommand {
    /// The domain to generate ssl certificates for
    #[arg(short, long)]
    domain: String,

    /// The path to the ssl certificates
    #[arg(short, long)]
    path: String,

    /// The path to the ssl key
    #[arg(short, long)]
    subject_alt_names: String,
}

pub async fn run(
    _command: &SslCommand,
    _client: &StoreClient<Etcd>,
    _traefik_config: &mut TraefikConfig,
) -> TraefikResult<()> {
    println!("TODO: implement ssl command");
    Ok(())
}
