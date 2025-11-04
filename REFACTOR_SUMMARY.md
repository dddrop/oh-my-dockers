# 重构总结

## 已完成的变更

### 1. 架构重构

从集中式配置管理转向分布式项目配置：

**之前：**
- 所有项目配置在 `~/.oh-my-dockers/projects/*.toml`
- 工具生成 docker-compose 文件
- 模板驱动的服务管理

**现在：**
- 每个项目目录下有 `omd.toml`
- 用户管理自己的 `docker-compose.yml`
- 工具仅负责 Caddy 配置和端口冲突检测

### 2. 新增模块

#### `src/registry.rs`
- 端口注册表管理
- 项目注册/注销
- 端口冲突检测
- 保存位置：`~/.oh-my-dockers/registry.json`

#### `src/docker_compose.rs`
- Docker Compose 文件解析器
- 提取服务名、容器名、端口映射
- 支持多种端口配置格式

#### `src/init.rs`
- `omd init` 命令实现
- 交互式配置文件生成

### 3. 修改的模块

#### `src/config.rs`
- 简化 `ProjectConfig` 结构
- 移除 `services`、`mode`、`port_offset` 字段
- 新增 `load_project_config()` 从当前目录读取
- 简化全局配置，移除模板相关配置

#### `src/project.rs`
- 完全重写 `up()`、`down()`、`list()` 命令
- `up()`: 读取本地配置 + 解析 docker-compose.yml + 端口冲突检测
- `down()`: 删除 Caddy 配置 + 注销项目
- `list()`: 从注册表显示所有项目

#### `src/project/caddy.rs`
- 使用解析的容器名生成 Caddy 配置
- 支持自动路由生成和自定义路由覆盖
- 移除 `auto_subdomains` 和 `managed` 模式逻辑

#### `src/main.rs`
- 新增 `Init` 命令
- 修改 `ProjectCommands` 不再需要项目名参数
- 移除 `Migrate` 命令

### 4. 删除的文件

- `src/project/compose.rs` - 不再生成 docker-compose 文件
- `src/migrate.rs` - 不再需要迁移功能

### 5. 依赖更新

在 `Cargo.toml` 中添加：
- `serde_yaml = "0.9"` - 解析 docker-compose.yml
- `serde_json = "1.0"` - 保存注册表
- `tempfile = "3"` - 测试用

### 6. 配置文件格式

#### 新的 `omd.toml` 格式

```toml
[project]
name = "my-project"
domain = "my-project.local"

[network]
name = "my-project-net"

[caddy.routes]
# 可选的自定义路由
api = "backend:3000"
app = "frontend:80"
```

#### 注册表格式 (`registry.json`)

```json
{
  "projects": {
    "my-project": {
      "name": "my-project",
      "path": "/path/to/my-project",
      "domain": "my-project.local",
      "network": "my-project-net",
      "ports": [5432, 3000],
      "containers": ["my-project-postgres", "my-project-api"]
    }
  }
}
```

### 7. 文档更新

已更新以下文档以反映新架构：
- `README.md` - 完全重写
- `docs/MANUAL.md` - 完全重写（英文）
- `docs/MANUAL.ja.md` - 完全重写（日文）
- `docs/MANUAL.zh.md` - 完全重写（中文）

### 8. 新的工作流程

```bash
# 1. 初始化项目
cd /path/to/project
omd init

# 2. 创建 docker-compose.yml
# （用户自己管理）

# 3. 配置基础设施
omd up
# - 检查端口冲突
# - 创建网络
# - 生成 Caddy 配置
# - 注册项目

# 4. 启动容器
docker compose up -d

# 5. 访问服务
# https://your-project.local
```

### 9. 主要优势

1. **更灵活**：用户完全控制 docker-compose.yml
2. **端口冲突检测**：自动防止多项目端口冲突
3. **简化配置**：不再需要复杂的模板系统
4. **本地化**：每个项目独立配置
5. **透明性**：用户可以看到并修改所有配置

### 10. 待办事项（可选）

- [ ] 添加集成测试
- [ ] 创建示例项目
- [ ] 添加迁移工具（从旧配置迁移到新配置）
- [ ] 改进错误消息
- [ ] 添加 shell 补全脚本

## 测试建议

1. **基本工作流程测试**
   ```bash
   mkdir test-project
   cd test-project
   omd init
   # 创建简单的 docker-compose.yml
   omd up
   docker compose up -d
   omd project list
   omd down
   docker compose down
   ```

2. **端口冲突测试**
   - 创建两个使用相同端口的项目
   - 验证第二个项目的 `omd up` 报告冲突

3. **自定义路由测试**
   - 在 `omd.toml` 中配置自定义路由
   - 验证生成的 Caddy 配置正确

## 迁移指南（针对现有用户）

如果您之前使用旧版本的 omd：

1. **备份现有配置**
   ```bash
   cp -r ~/.oh-my-dockers ~/.oh-my-dockers.backup
   ```

2. **对每个项目**
   ```bash
   cd /path/to/your/project
   
   # 初始化新配置
   omd init
   
   # 如果项目使用 managed 模式，需要创建 docker-compose.yml
   # 可以参考 ~/.oh-my-dockers.backup/generated/ 中的文件
   
   # 重新配置
   omd up
   ```

3. **清理旧配置**（可选）
   ```bash
   rm -rf ~/.oh-my-dockers.backup
   ```

## 文件清单

### 新增
- `src/registry.rs`
- `src/docker_compose.rs`
- `src/init.rs`

### 修改
- `src/config.rs`
- `src/main.rs`
- `src/project.rs`
- `src/project/caddy.rs`
- `src/ports.rs`
- `src/network.rs`
- `Cargo.toml`
- `README.md`
- `docs/MANUAL.md`
- `docs/MANUAL.ja.md`
- `docs/MANUAL.zh.md`

### 删除
- `src/project/compose.rs`
- `src/migrate.rs`

---

重构完成日期：2025-11-04

