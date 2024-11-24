use sqlx::{MySql, Pool};
use std::collections::HashMap;

use crate::{
    config::{
        deployment::{DeploymentConfig, DeploymentProtocol},
        host::{HostConfig, PathConfig},
        selections::{FromClientIpConfig, SelectionConfig, WithCookieConfig},
    },
    error::TraefikResult,
};

#[derive(sqlx::FromRow, Debug)]
pub struct HostRow {
    pub id: i64,
    pub traefik_config_id: i64,
    pub domain: String,
}

#[derive(sqlx::FromRow, Debug)]
pub struct PathRow {
    pub id: i64,
    pub host_id: i64,
    pub path: String,
    pub strip_prefix: bool,
    pub pass_through: bool,
}

#[derive(sqlx::FromRow, Debug)]
pub struct DeploymentRow {
    pub id: i64,
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub weight: i64,
    pub protocol_id: i64,
    pub host_id: Option<i64>,
    pub path_id: Option<i64>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct MiddlewareRow {
    pub id: i64,
    pub name: String,
}

#[derive(sqlx::FromRow, Debug)]
pub struct SelectionRow {
    pub id: i64,
    pub host_id: Option<i64>,
    pub deployment_id: Option<i64>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct WithCookieRow {
    pub id: i64,
    pub selection_config_id: i64,
    pub name: String,
    pub value: Option<String>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct FromClientIpRow {
    pub id: i64,
    pub selection_config_id: i64,
    pub range: Option<String>,
    pub ip: Option<String>,
}

#[derive(Debug)]
pub struct DeploymentWithRelations {
    pub deployment: DeploymentRow,
    pub selection: Option<SelectionWithRelations>,
    pub middlewares: Vec<MiddlewareRow>,
}

#[derive(Debug)]
pub struct SelectionWithRelations {
    pub selection: SelectionRow,
    pub with_cookie: Option<WithCookieRow>,
    pub from_client_ip: Option<FromClientIpRow>,
}

#[derive(Debug)]
pub struct CompleteHostRow {
    pub host: HostRow,
    pub paths: Vec<(PathRow, Vec<DeploymentWithRelations>, Vec<MiddlewareRow>)>,
    pub deployments: Vec<DeploymentWithRelations>,
    pub middlewares: Vec<MiddlewareRow>,
    pub selection: Option<SelectionWithRelations>,
}

impl From<i64> for DeploymentProtocol {
    fn from(value: i64) -> Self {
        match value {
            1 => DeploymentProtocol::Http,
            2 => DeploymentProtocol::Https,
            3 => DeploymentProtocol::Tcp,
            _ => DeploymentProtocol::Invalid,
        }
    }
}

pub async fn convert_to_host_config(
    pool: &Pool<MySql>,
    host: HostRow,
) -> TraefikResult<Option<HostConfig>> {
    let (paths_future, deployments_future, middlewares_future, selection_future) = tokio::join!(
        async {
            let paths: Vec<PathRow> =
                sqlx::query_as::<_, PathRow>("SELECT * FROM paths WHERE host_id = ?")
                    .bind(host.id)
                    .fetch_all(pool)
                    .await?;

            let mut path_data = Vec::new();
            for path in paths {
                let (deployments, middlewares) = tokio::join!(
                    async {
                        let deployments = sqlx::query_as::<_, DeploymentRow>(
                            "SELECT * FROM deployments WHERE path_id = ?",
                        )
                        .bind(path.id)
                        .fetch_all(pool)
                        .await?;

                        let mut deployments_with_relations = Vec::new();
                        for d in deployments {
                            let (selection, middlewares) = tokio::join!(
                                async {
                                    let selection = sqlx::query_as::<_, SelectionRow>(
                                        "SELECT * FROM selection_configs WHERE deployment_id = ?",
                                    )
                                    .bind(d.id)
                                    .fetch_optional(pool)
                                    .await?;

                                    if let Some(selection) = selection {
                                        let (cookie, client_ip) = tokio::join!(
                                            sqlx::query_as::<_, WithCookieRow>(
                                                "SELECT * FROM with_cookie_configs WHERE selection_config_id = ?"
                                            )
                                            .bind(selection.id)
                                            .fetch_optional(pool),
                                            sqlx::query_as::<_, FromClientIpRow>(
                                                "SELECT * FROM from_client_ip_configs WHERE selection_config_id = ?"
                                            )
                                            .bind(selection.id)
                                            .fetch_optional(pool)
                                        );

                                        Ok::<_, sqlx::Error>(Some(SelectionWithRelations {
                                            selection,
                                            with_cookie: cookie?,
                                            from_client_ip: client_ip?,
                                        }))
                                    } else {
                                        Ok(None)
                                    }
                                },
                                sqlx::query_as::<_, MiddlewareRow>(
                                    r#"SELECT m.* FROM middlewares m
                                    JOIN deployment_middlewares dm ON m.id = dm.middleware_id
                                    WHERE dm.deployment_id = ?"#
                                )
                                .bind(d.id)
                                .fetch_all(pool)
                            );

                            deployments_with_relations.push(DeploymentWithRelations {
                                deployment: d,
                                selection: selection?,
                                middlewares: middlewares?,
                            });
                        }
                        Ok::<_, sqlx::Error>(deployments_with_relations)
                    },
                    sqlx::query_as::<_, MiddlewareRow>(
                        r#"SELECT m.* FROM middlewares m 
                        JOIN path_middlewares pm ON m.id = pm.middleware_id 
                        WHERE pm.path_id = ?"#
                    )
                    .bind(path.id)
                    .fetch_all(pool)
                );
                path_data.push((path, deployments?, middlewares?));
            }
            Ok::<_, sqlx::Error>(path_data)
        },
        async {
            let deployments =
                sqlx::query_as::<_, DeploymentRow>("SELECT * FROM deployments WHERE host_id = ?")
                    .bind(host.id)
                    .fetch_all(pool)
                    .await?;

            let mut deployments_with_relations = Vec::new();
            for d in deployments {
                let (selection, middlewares) = tokio::join!(
                    async {
                        let selection = sqlx::query_as::<_, SelectionRow>(
                            "SELECT * FROM selection_configs WHERE deployment_id = ?",
                        )
                        .bind(d.id)
                        .fetch_optional(pool)
                        .await?;

                        if let Some(selection) = selection {
                            let (cookie, client_ip) = tokio::join!(
                                sqlx::query_as::<_, WithCookieRow>(
                                    "SELECT * FROM with_cookie_configs WHERE selection_config_id = ?"
                                )
                                .bind(selection.id)
                                .fetch_optional(pool),
                                sqlx::query_as::<_, FromClientIpRow>(
                                    "SELECT * FROM from_client_ip_configs WHERE selection_config_id = ?"
                                )
                                .bind(selection.id)
                                .fetch_optional(pool)
                            );

                            Ok::<_, sqlx::Error>(Some(SelectionWithRelations {
                                selection,
                                with_cookie: cookie?,
                                from_client_ip: client_ip?,
                            }))
                        } else {
                            Ok(None)
                        }
                    },
                    sqlx::query_as::<_, MiddlewareRow>(
                        r#"SELECT m.* FROM middlewares m
                        JOIN deployment_middlewares dm ON m.id = dm.middleware_id
                        WHERE dm.deployment_id = ?"#
                    )
                    .bind(d.id)
                    .fetch_all(pool)
                );

                deployments_with_relations.push(DeploymentWithRelations {
                    deployment: d,
                    selection: selection?,
                    middlewares: middlewares?,
                });
            }
            Ok::<_, sqlx::Error>(deployments_with_relations)
        },
        sqlx::query_as::<_, MiddlewareRow>(
            r#"SELECT m.* FROM middlewares m 
            JOIN host_middlewares hm ON m.id = hm.middleware_id 
            WHERE hm.host_id = ?"#
        )
        .bind(host.id)
        .fetch_all(pool),
        async {
            let selection = sqlx::query_as::<_, SelectionRow>(
                "SELECT * FROM selection_configs WHERE host_id = ?",
            )
            .bind(host.id)
            .fetch_optional(pool)
            .await?;

            if let Some(selection) = selection {
                let (cookie, client_ip) = tokio::join!(
                    sqlx::query_as::<_, WithCookieRow>(
                        "SELECT * FROM with_cookie_configs WHERE selection_config_id = ?"
                    )
                    .bind(selection.id)
                    .fetch_optional(pool),
                    sqlx::query_as::<_, FromClientIpRow>(
                        "SELECT * FROM from_client_ip_configs WHERE selection_config_id = ?"
                    )
                    .bind(selection.id)
                    .fetch_optional(pool)
                );

                Ok::<_, sqlx::Error>(Some(SelectionWithRelations {
                    selection,
                    with_cookie: cookie?,
                    from_client_ip: client_ip?,
                }))
            } else {
                Ok(None)
            }
        }
    );

    Ok(Some(HostConfig::try_from(CompleteHostRow {
        host,
        paths: paths_future?,
        deployments: deployments_future?,
        middlewares: middlewares_future?,
        selection: selection_future?,
    })?))
}

impl TryFrom<CompleteHostRow> for HostConfig {
    type Error = sqlx::Error;

    fn try_from(complete_row: CompleteHostRow) -> Result<Self, Self::Error> {
        let CompleteHostRow {
            host,
            paths,
            deployments,
            middlewares,
            selection,
        } = complete_row;

        let paths: Vec<PathConfig> = paths
            .into_iter()
            .map(|(path_row, path_deployments, path_middlewares)| {
                let deployments: HashMap<String, DeploymentConfig> = path_deployments
                    .into_iter()
                    .map(|d| {
                        (
                            d.deployment.name.clone(),
                            DeploymentConfig {
                                name: d.deployment.name,
                                ip: d.deployment.ip,
                                port: d.deployment.port,
                                weight: d.deployment.weight as usize,
                                protocol: DeploymentProtocol::from(d.deployment.protocol_id),
                                selection: d.selection.map(|s| SelectionConfig {
                                    with_cookie: s.with_cookie.map(|c| WithCookieConfig {
                                        name: c.name,
                                        value: c.value,
                                    }),
                                    from_client_ip: s.from_client_ip.map(|ip| FromClientIpConfig {
                                        range: ip.range,
                                        ip: ip.ip,
                                    }),
                                }),
                                middlewares: Some(
                                    d.middlewares.into_iter().map(|m| m.name).collect(),
                                ),
                            },
                        )
                    })
                    .collect();

                PathConfig {
                    path: path_row.path,
                    deployments,
                    middlewares: path_middlewares.into_iter().map(|m| m.name).collect(),
                    strip_prefix: path_row.strip_prefix,
                    pass_through: path_row.pass_through,
                }
            })
            .collect();

        let host_deployments: HashMap<String, DeploymentConfig> = deployments
            .into_iter()
            .map(|d| {
                (
                    d.deployment.name.clone(),
                    DeploymentConfig {
                        name: d.deployment.name,
                        ip: d.deployment.ip,
                        port: d.deployment.port,
                        weight: d.deployment.weight as usize,
                        protocol: DeploymentProtocol::from(d.deployment.protocol_id),
                        selection: d.selection.map(|s| SelectionConfig {
                            with_cookie: s.with_cookie.map(|c| WithCookieConfig {
                                name: c.name,
                                value: c.value,
                            }),
                            from_client_ip: s.from_client_ip.map(|ip| FromClientIpConfig {
                                range: ip.range,
                                ip: ip.ip,
                            }),
                        }),
                        middlewares: Some(d.middlewares.into_iter().map(|m| m.name).collect()),
                    },
                )
            })
            .collect();

        let selection_config: Option<SelectionConfig> = selection.map(|s| SelectionConfig {
            with_cookie: s.with_cookie.map(|c| WithCookieConfig {
                name: c.name,
                value: c.value,
            }),
            from_client_ip: s.from_client_ip.map(|ip| FromClientIpConfig {
                range: ip.range,
                ip: ip.ip,
            }),
        });

        Ok(HostConfig {
            domain: host.domain,
            paths,
            deployments: host_deployments,
            middlewares: middlewares.into_iter().map(|m| m.name).collect(),
            selection: selection_config,
        })
    }
}
