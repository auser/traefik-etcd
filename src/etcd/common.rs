// #[derive(Serialize, Deserialize, Debug)]
// pub struct Config {
//     pub global: GlobalConfig,
//     pub middlewares: HashMap<String, MiddlewareConfig>,
//     pub services: HashMap<String, ServiceConfig>,
//     pub hosts: Vec<HostConfig>,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct HostConfig {
//     pub domain: String,
//     pub backend: Option<BackendConfig>,
//     pub entrypoints: Vec<String>,
//     pub middlewares: Vec<String>,
//     #[serde(default)]
//     pub include_redirect: bool,
//     #[serde(default)]
//     pub www_redirect: bool,
//     #[serde(default)]
//     pub paths: Vec<PathConfig>,
//     #[serde(default)]
//     pub deployment: DeploymentConfig,
//     #[serde(default)]
//     pub test_cookie: Option<TestCookieConfig>,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct LoadBalancerConfig {
//     pub(crate) servers: Vec<ServerConfig>,
//     pub(crate) pass_host_header: bool,
//     pub(crate) response_forwarding: Option<ResponseForwardingConfig>,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct ServerConfig {
//     pub(crate) url: String,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct ResponseForwardingConfig {
//     pub(crate) allowed_hosts: Vec<String>,
// }
