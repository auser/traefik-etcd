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
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(255) NOT NULL,
    config JSON NOT NULL,
    created_at TIMESTAMP(6) DEFAULT CURRENT_TIMESTAMP(6),
    updated_at TIMESTAMP(6) DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
    version INT NOT NULL DEFAULT 1,
    INDEX idx_name (name),
    INDEX idx_created_at (created_at)
) ENGINE=InnoDB;