-- Create enum table for deployment protocols

CREATE TABLE IF NOT EXISTS deployment_protocols (
    id INT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(255) NOT NULL UNIQUE
) ENGINE=InnoDB;


INSERT INTO deployment_protocols (name) 
VALUES ('http'),
       ('https'),
       ('tcp'),
       ('invalid');

-- Create table for TraefikConfig
CREATE TABLE IF NOT EXISTS traefik_configs (
    id INT PRIMARY KEY AUTO_INCREMENT,
    rule_prefix VARCHAR(255) NOT NULL DEFAULT 'Host'
);

-- Create table for HostConfig
CREATE TABLE IF NOT EXISTS hosts (
    id INT PRIMARY KEY AUTO_INCREMENT,
    traefik_config_id INT NOT NULL,
    domain VARCHAR(255) NOT NULL,
    FOREIGN KEY (traefik_config_id) REFERENCES traefik_configs(id) ON DELETE CASCADE
);

-- Create table for PathConfig
CREATE TABLE IF NOT EXISTS paths (
    id INT PRIMARY KEY AUTO_INCREMENT,
    host_id INT NOT NULL,
    path VARCHAR(255) NOT NULL,
    strip_prefix BOOLEAN NOT NULL DEFAULT FALSE,
    pass_through BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (host_id) REFERENCES hosts(id) ON DELETE CASCADE
);

-- Create table for DeploymentConfig
CREATE TABLE IF NOT EXISTS deployments (
    id INT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(255) NOT NULL DEFAULT '',
    ip VARCHAR(255) NOT NULL DEFAULT '127.0.0.1',
    port INT NOT NULL DEFAULT 8080,
    weight INT NOT NULL DEFAULT 1,
    protocol_id INT NOT NULL,
    host_id INT,
    path_id INT,
    FOREIGN KEY (protocol_id) REFERENCES deployment_protocols(id),
    FOREIGN KEY (host_id) REFERENCES hosts(id) ON DELETE CASCADE,
    FOREIGN KEY (path_id) REFERENCES paths(id) ON DELETE CASCADE,
    CONSTRAINT check_deployment_parent CHECK (
        (host_id IS NOT NULL AND path_id IS NULL) OR 
        (host_id IS NULL AND path_id IS NOT NULL)
    )
);

-- Create table for SelectionConfig
CREATE TABLE IF NOT EXISTS selection_configs (
    id INT PRIMARY KEY AUTO_INCREMENT,
    host_id INT,
    deployment_id INT,
    FOREIGN KEY (host_id) REFERENCES hosts(id) ON DELETE CASCADE,
    FOREIGN KEY (deployment_id) REFERENCES deployments(id) ON DELETE CASCADE,
    CONSTRAINT check_selection_parent CHECK (
        (host_id IS NOT NULL AND deployment_id IS NULL) OR 
        (host_id IS NULL AND deployment_id IS NOT NULL)
    )
);

-- Create table for WithCookieConfig
CREATE TABLE IF NOT EXISTS with_cookie_configs (
    id INT PRIMARY KEY AUTO_INCREMENT,
    selection_config_id INT NOT NULL,
    name VARCHAR(255) NOT NULL,
    value VARCHAR(255),
    FOREIGN KEY (selection_config_id) REFERENCES selection_configs(id) ON DELETE CASCADE
);

-- Create table for FromClientIpConfig
CREATE TABLE IF NOT EXISTS from_client_ip_configs (
    id INT PRIMARY KEY AUTO_INCREMENT,
    selection_config_id INT NOT NULL,
    ip_range VARCHAR(255),
    ip VARCHAR(255),
    FOREIGN KEY (selection_config_id) REFERENCES selection_configs(id) ON DELETE CASCADE
);

-- Create table for MiddlewareConfig
CREATE TABLE IF NOT EXISTS middlewares (
    id INT PRIMARY KEY AUTO_INCREMENT,
    traefik_config_id INT NOT NULL,
    name VARCHAR(255) NOT NULL DEFAULT '',
    protocol VARCHAR(255) NOT NULL DEFAULT 'http',
    FOREIGN KEY (traefik_config_id) REFERENCES traefik_configs(id) ON DELETE CASCADE
);

-- Create table for HeadersConfig
CREATE TABLE IF NOT EXISTS headers_configs (
    id INT PRIMARY KEY AUTO_INCREMENT,
    middleware_id INT NOT NULL UNIQUE,
    add_vary_header BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (middleware_id) REFERENCES middlewares(id) ON DELETE CASCADE
);

-- Create tables for header key-value pairs
CREATE TABLE IF NOT EXISTS custom_request_headers (
    id INT PRIMARY KEY AUTO_INCREMENT,
    headers_config_id INT NOT NULL,
    header_key VARCHAR(255) NOT NULL,
    header_value VARCHAR(255) NOT NULL,
    FOREIGN KEY (headers_config_id) REFERENCES headers_configs(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS custom_response_headers (
    id INT PRIMARY KEY AUTO_INCREMENT,
    headers_config_id INT NOT NULL,
    header_key VARCHAR(255) NOT NULL,
    header_value VARCHAR(255) NOT NULL,
    FOREIGN KEY (headers_config_id) REFERENCES headers_configs(id) ON DELETE CASCADE
);

-- Create tables for header arrays
CREATE TABLE IF NOT EXISTS access_control_allow_methods (
    id INT PRIMARY KEY AUTO_INCREMENT,
    headers_config_id INT NOT NULL,
    method VARCHAR(255) NOT NULL,
    FOREIGN KEY (headers_config_id) REFERENCES headers_configs(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS access_control_allow_headers (
    id INT PRIMARY KEY AUTO_INCREMENT,
    headers_config_id INT NOT NULL,
    header VARCHAR(255) NOT NULL,
    FOREIGN KEY (headers_config_id) REFERENCES headers_configs(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS access_control_expose_headers (
    id INT PRIMARY KEY AUTO_INCREMENT,
    headers_config_id INT NOT NULL,
    header VARCHAR(255) NOT NULL,
    FOREIGN KEY (headers_config_id) REFERENCES headers_configs(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS access_control_allow_origin_list (
    id INT PRIMARY KEY AUTO_INCREMENT,
    headers_config_id INT NOT NULL,
    origin VARCHAR(255) NOT NULL,
    FOREIGN KEY (headers_config_id) REFERENCES headers_configs(id) ON DELETE CASCADE
);

-- Create junction table for hosts and middlewares
CREATE TABLE IF NOT EXISTS host_middlewares (
    host_id INT NOT NULL,
    middleware_id INT NOT NULL,
    PRIMARY KEY (host_id, middleware_id),
    FOREIGN KEY (host_id) REFERENCES hosts(id) ON DELETE CASCADE,
    FOREIGN KEY (middleware_id) REFERENCES middlewares(id) ON DELETE CASCADE
);

-- Create junction table for paths and middlewares
CREATE TABLE IF NOT EXISTS path_middlewares (
    path_id INT NOT NULL,
    middleware_id INT NOT NULL,
    PRIMARY KEY (path_id, middleware_id),
    FOREIGN KEY (path_id) REFERENCES paths(id) ON DELETE CASCADE,
    FOREIGN KEY (middleware_id) REFERENCES middlewares(id) ON DELETE CASCADE
);

-- Create junction table for deployments and middlewares
CREATE TABLE IF NOT EXISTS deployment_middlewares (
    deployment_id INT NOT NULL,
    middleware_id INT NOT NULL,
    PRIMARY KEY (deployment_id, middleware_id),
    FOREIGN KEY (deployment_id) REFERENCES deployments(id) ON DELETE CASCADE,
    FOREIGN KEY (middleware_id) REFERENCES middlewares(id) ON DELETE CASCADE
);

-- Create table for HealthCheckConfig
CREATE TABLE IF NOT EXISTS health_checks (
    id INT PRIMARY KEY AUTO_INCREMENT,
    deployment_id INT NOT NULL UNIQUE,
    path VARCHAR(255) NOT NULL,
    check_interval VARCHAR(255) NOT NULL,
    check_timeout VARCHAR(255) NOT NULL,
    FOREIGN KEY (deployment_id) REFERENCES deployments(id) ON DELETE CASCADE
);