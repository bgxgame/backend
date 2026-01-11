# 项目深度总结

## 项目概述
这是一个基于 Rust 开发的生活计划管理系统后端，采用 Axum + Tokio 异步框架构建。项目名为"My-Life-Planner"，提供用户认证、项目管理和任务跟踪功能。

## 技术栈
- **编程语言**: Rust
- **Web框架**: Axum (现代异步 Rust Web 框架)
- **异步运行时**: Tokio
- **数据库**: PostgreSQL (使用 SQLx 异步驱动)
- **密码哈希**: Argon2 (安全的密码加密算法)
- **身份认证**: JWT (JSON Web Tokens) + Refresh Token 机制
- **数据序列化**: Serde
- **日志追踪**: Tracing
- **输入验证**: Validator
- **错误处理**: ThisError

## 项目架构

### 1. 核心模块
- **用户系统**: 用户注册/登录，使用 Argon2 加密密码，JWT 认证
- **项目管理**: 对应 `projects` 表，支持创建、修改、删除项目
- **任务管理**: 对应 `issues` 表，任务属于项目，支持优先级、状态管理
- **评论系统**: 任务可以添加评论
- **统一搜索**: 支持跨项目和任务的全文搜索

### 2. 文件结构
- `main.rs`: 项目入口点，设置路由、中间件和启动服务器
- `models.rs`: 数据模型定义，包含 Project、Issue、User 等实体
- `handlers.rs`: 所有 API 控制器逻辑
- `auth.rs`: 认证相关功能，包括 JWT、密码哈希等
- `error.rs`: 统一错误处理
- `validation.rs`: 输入验证逻辑

### 3. 数据库设计
- **users 表**: 存储用户信息（ID、用户名、密码哈希）
- **projects 表**: 存储项目信息（ID、用户ID、名称、描述、状态、颜色等）
- **issues 表**: 存储任务信息（ID、项目ID、用户ID、标题、描述、状态、优先级等）
- **comments 表**: 存储评论信息
- **refresh_tokens 表**: 存储刷新令牌用于 JWT 无感刷新

### 4. API 接口
- **认证接口**:
  - `POST /api/register`: 用户注册
  - `POST /api/login`: 用户登录
  - `POST /api/refresh`: 刷新令牌

- **项目接口**:
  - `GET /api/projects`: 获取用户所有项目
  - `POST /api/projects`: 创建项目
  - `PATCH /api/projects/:id`: 更新项目
  - `DELETE /api/projects/:id`: 删除项目

- **任务接口**:
  - `GET /api/issues`: 获取用户所有任务
  - `GET /api/projects/:id/issues`: 获取特定项目下的任务
  - `POST /api/issues`: 创建任务
  - `PATCH /api/issues/:id`: 更新任务
  - `DELETE /api/issues/:id`: 删除任务
  - `GET /api/search`: 统一搜索

- **评论接口**:
  - `GET /api/issues/:id/comments`: 获取任务评论
  - `POST /api/issues/:id/comments`: 添加评论

### 5. 安全特性
- **密码安全**: 使用 Argon2 算法加密用户密码
- **身份认证**: JWT 令牌认证，15分钟有效期
- **权限控制**: 所有敏感操作都需要有效的 JWT 令牌
- **刷新令牌**: 支持 Refresh Token 机制，实现无感刷新
- **输入验证**: 使用 Validator 库对所有输入进行验证
- **访问控制**: 确保用户只能访问自己拥有的资源

### 6. 运行环境
- **服务端口**: 3000
- **数据库**: PostgreSQL
- **配置**: 通过 `.env` 文件管理环境变量
- **日志**: 使用 Tracing 进行详细的运行时日志追踪

### 7. 特殊功能
- **统一搜索**: 支持跨项目和任务的全文搜索功能
- **优先级管理**: 任务支持不同优先级设置
- **状态管理**: 项目和任务都有状态字段
- **颜色标识**: 项目支持颜色标签
- **截止日期**: 任务支持设置截止日期

## 项目特点
1. **安全性强**: 采用最新的安全实践，包括 Argon2 密码哈希和 JWT 认证
2. **性能优秀**: 基于 Rust 和异步架构，具有出色的性能表现
3. **功能完整**: 包含完整的用户认证、项目管理、任务跟踪等功能
4. **易于扩展**: 架构清晰，便于后续功能扩展
5. **用户体验佳**: 支持无感刷新令牌，提供流畅的用户体验

这是一个设计良好的现代化后端项目，采用了最佳实践，具备了生产就绪的安全性和性能特征。