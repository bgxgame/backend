-- Add up migration script here
CREATE TABLE users (
    id SERIAL PRIMARY KEY,                     -- 自增主键
    username VARCHAR(50) NOT NULL UNIQUE,      -- 用户名，必须唯一
    password_hash VARCHAR(255) NOT NULL,       -- 密码哈希 (严禁明文存储，推荐 Argon2)
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP -- 创建时间
);

CREATE TABLE plans (
    id SERIAL PRIMARY KEY,

    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE, -- 关联用户

    -- 核心内容
    title VARCHAR(150) NOT NULL,               -- 计划标题
    description TEXT,                          -- 详细描述 (支持 Markdown 更好)
    
    -- 状态管理 (建议前端做枚举映射)
    -- values: 'pending'(待办), 'in_progress'(进行中), 'completed'(已完成), 'archived'(归档)
    status VARCHAR(20) NOT NULL DEFAULT 'pending', 
    
    -- 分类与优先级
    category VARCHAR(50) DEFAULT 'life',       -- 例如: 'work', 'life', 'study'
    priority INTEGER DEFAULT 0,                -- 优先级: 0(普通), 1(重要), 2(紧急)
    
    -- 时间管理
    due_date TIMESTAMP WITH TIME ZONE,         -- 截止日期 (可为空)
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- 权限控制
    -- 如果是 false，只有登录后可见；如果是 true，游客可见
    is_public BOOLEAN NOT NULL DEFAULT TRUE    
);
-- 建议添加索引以提高查询效率
CREATE INDEX idx_plans_user_id ON plans(user_id);