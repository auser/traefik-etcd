use clap::Args;

use crate::{core::client::StoreClient, error::TraefikResult, features::etcd::Etcd, TraefikConfig};

#[derive(Args, Debug)]
pub struct GraphCommand {
    /// Output the graph in DOT format
    #[arg(short, long)]
    pub dot: bool,
}

pub async fn run(
    command: &GraphCommand,
    _client: &StoreClient<Etcd>,
    traefik_config: &mut TraefikConfig,
) -> TraefikResult<()> {
    match traefik_config.into_graph(command.dot) {
        Ok((graph, dot_graph)) => {
            if command.dot {
                println!("{}", dot_graph.unwrap());
            } else {
                println!("{:?}", graph);
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    Ok(())
}
