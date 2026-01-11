-- 1. 清理旧表
DROP TABLE IF EXISTS issues;
DROP TABLE IF EXISTS projects;
DROP TABLE IF EXISTS users;

-- 2. 创建用户表
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 3. 创建项目表 (Project)
CREATE TABLE projects (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    
    -- 项目状态: backlog(积压), active(激活), completed(完成), paused(暂停), canceled(取消)
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    
    -- 视觉标识 (Linear 风格常用)
    color VARCHAR(7) DEFAULT '#5E6AD2', -- 项目主题色
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 4. 创建任务表 (Issue)
CREATE TABLE issues (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE, -- 创建者/负责人
    
    title VARCHAR(255) NOT NULL,
    description TEXT, -- 支持 Markdown
    
    -- 任务状态: backlog, todo, in_progress, done, canceled
    status VARCHAR(20) NOT NULL DEFAULT 'todo',
    
    -- 优先级: 0(无), 1(低), 2(中), 3(高), 4(紧急)
    priority INTEGER NOT NULL DEFAULT 0,
    
    due_date TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 5. 创建索引提高查询效率
CREATE INDEX idx_projects_user_id ON projects(user_id);
CREATE INDEX idx_issues_project_id ON issues(project_id);
CREATE INDEX idx_issues_user_id ON issues(user_id);

-- 6. 自动更新 updated_at 的触发器函数 (PostgreSQL 特色)
CREATE OR REPLACE FUNCTION update_modified_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_projects_modtime BEFORE UPDATE ON projects FOR EACH ROW EXECUTE PROCEDURE update_modified_column();
CREATE TRIGGER update_issues_modtime BEFORE UPDATE ON issues FOR EACH ROW EXECUTE PROCEDURE update_modified_column();


CREATE TABLE refresh_tokens (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE comments (
    id SERIAL PRIMARY KEY,
    issue_id INTEGER NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 索引提高查询效率
CREATE INDEX idx_comments_issue_id ON comments(issue_id);