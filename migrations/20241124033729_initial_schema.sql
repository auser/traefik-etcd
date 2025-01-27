CREATE TABLE IF NOT EXISTS schema_migrations (
    version BIGINT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    applied_at TIMESTAMP(6) DEFAULT CURRENT_TIMESTAMP(6)
) ENGINE=InnoDB;

-- 20241125000001_create_deployment_protocols.sql
CREATE TABLE IF NOT EXISTS deployment_protocols (
    id SMALLINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(10) NOT NULL UNIQUE,
    created_at TIMESTAMP(6) DEFAULT CURRENT_TIMESTAMP(6)
) ENGINE=InnoDB;

INSERT INTO deployment_protocols (name) 
VALUES ('http'),
       ('https'),
       ('tcp'),
       ('invalid');

-- 20241125000002_create_config_versions.sql
CREATE TABLE IF NOT EXISTS config_versions (
    id BIGINT NOT NULL AUTO_INCREMENT,  -- Change from BIGINT UNSIGNED to BIGINT
    name VARCHAR(255) NOT NULL,
    config JSON NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    version INT NOT NULL DEFAULT 1,
    PRIMARY KEY (id),
    INDEX idx_name (name),
    INDEX idx_created_at (created_at)
);

-- Add version_history table
CREATE TABLE IF NOT EXISTS config_version_history (
    id BIGINT NOT NULL AUTO_INCREMENT,
    config_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    config JSON NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    version INT NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (config_id) REFERENCES config_versions(id) ON DELETE CASCADE
);
