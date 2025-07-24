-- 创建用户表
CREATE TABLE users (
    id VARCHAR(36) PRIMARY KEY,
    user_name VARCHAR(10) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

-- 创建索引
CREATE INDEX idx_users_email ON users(email);