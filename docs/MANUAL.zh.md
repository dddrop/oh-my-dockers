# omd 使用手册

omd CLI 工具完整参考指南。

## 目录

1. [简介](#简介)
2. [安装](#安装)
3. [快速开始](#快速开始)
4. [配置](#配置)
5. [命令参考](#命令参考)
6. [项目配置](#项目配置)
7. [高级用法](#高级用法)
8. [故障排除](#故障排除)
9. [最佳实践](#最佳实践)

## 简介

omd（oh-my-dockers）是一个用于管理 Docker 开发环境的综合性 CLI 工具。它提供：

- **本地项目配置**：每个项目管理自己的 `omd.toml` 配置文件
- **自动网络管理**：自动创建和管理 Docker 网络
- **端口冲突检测**：防止多个项目之间的端口冲突
- **反向代理集成**：为 HTTPS 访问生成 Caddy 配置
- **智能容器发现**：自动解析 `docker-compose.yml` 获取容器信息
- **集中注册表**：跟踪所有项目及其端口分配

### 核心概念

**基于项目的工作流**：与集中式配置系统不同，omd 使用本地项目目录工作。每个项目都有自己的 `omd.toml` 配置文件。

**端口注册表**：omd 维护一个全局注册表（`~/.oh-my-dockers/registry.json`），跟踪哪些端口被哪些项目使用，防止冲突。

**Caddy 集成**：自动生成反向代理配置，允许您通过自定义域名使用 HTTPS 访问服务。

## 安装

### 系统要求

- **操作系统**：macOS、Linux 或 Windows（WSL2）
- **Docker**：版本 20.10 或更高
- **Docker Compose**：版本 2.0 或更高
- **Rust**：1.70 或更高（从源码构建时需要）
- **Caddy**：最新版本（用于反向代理功能）

### 从源码构建

```bash
# 克隆仓库
git clone <repository-url>
cd oh-my-dockers

# 构建发布版二进制文件
cargo build --release

# 二进制文件位于 target/release/omd
```

### 安装到系统 PATH

```bash
# 全局安装
cargo install --path .

# 验证安装
omd --help
```

## 快速开始

### 基本工作流程

```bash
# 1. 进入项目目录
cd /path/to/your/project

# 2. 初始化 omd 配置
omd init

# 3. 确保 docker-compose.yml 存在
# （你现有的 docker-compose.yml 文件）

# 4. 配置基础设施（网络、Caddy 等）
omd up

# 5. 启动容器
docker compose up -d

# 6. 访问服务
# https://your-project.local
```

### 完整示例

```bash
# 创建新项目
mkdir my-api
cd my-api

# 初始化 omd
omd init
# Project name [my-api]: 
# Domain [my-api.local]: 
# Network name [my-api-net]: 
# Do you want to configure Caddy routes now? [y/N]: n

# 创建 docker-compose.yml
cat > docker-compose.yml <<EOF
services:
  postgres:
    image: postgres:15
    container_name: my-api-postgres
    ports:
      - "5432:5432"
    environment:
      POSTGRES_PASSWORD: secret
    networks:
      - my-api-net

  api:
    build: .
    container_name: my-api-backend
    ports:
      - "3000:3000"
    depends_on:
      - postgres
    networks:
      - my-api-net

networks:
  my-api-net:
EOF

# 配置基础设施
omd up
# ℹ 正在解析 docker-compose.yml...
# ℹ 发现主机端口: 5432, 3000
# ✓ 无端口冲突
# ✓ 网络已创建
# ✓ Caddy 配置已生成
# ✓ 项目 my-api 配置完成！

# 启动服务
docker compose up -d

# 通过 https://api.my-api.local 访问
```

## 配置

### 配置目录

工具使用 `~/.oh-my-dockers` 作为默认配置目录。

**目录结构：**

```
~/.oh-my-dockers/
├── config.toml          # 全局设置
├── registry.json        # 项目注册表及端口分配
└── caddy/
    ├── Caddyfile        # 主 Caddy 配置
    ├── certs/           # SSL 证书
    └── projects/        # 生成的项目级 Caddy 配置
```

**自定义配置目录：**

```bash
export OH_MY_DOCKERS_DIR="/custom/path"
```

### 全局配置

全局配置文件位于 `~/.oh-my-dockers/config.toml`：

```toml
[global]
# Caddy 网络名称
caddy_network = "caddy-net"

# 目录（相对于配置目录）
caddy_projects_dir = "caddy/projects"
caddy_certs_dir = "caddy/certs"

[defaults]
# 默认时区
timezone = "Asia/Tokyo"

# 网络定义
[networks]
# Caddy 反向代理网络
caddy-net = {}
```

## 命令参考

### omd init

在当前目录初始化 `omd.toml` 配置。

```bash
omd init
```

**交互式提示：**
- 项目名称（默认：当前目录名）
- 域名（默认：`{项目名}.local`）
- 网络名称（默认：`{项目名}-net`）
- 配置 Caddy 路由（可选）

**输出**：在当前目录创建 `omd.toml`。

### omd project up

配置项目基础设施（从项目目录运行）。

```bash
cd /path/to/project
omd up
```

**执行内容：**
1. 从当前目录读取 `omd.toml`
2. 解析 `docker-compose.yml` 提取端口和容器名
3. 检查与其他已注册项目的端口冲突
4. 如果不存在则创建 Docker 网络
5. 生成 Caddy 反向代理配置
6. 在全局注册表中注册项目

**重要**：这不会启动容器。需要单独运行 `docker compose up -d`。

### omd project down

删除项目配置（从项目目录运行）。

```bash
cd /path/to/project
omd down
```

**执行内容：**
1. 从当前目录读取 `omd.toml`
2. 删除 Caddy 配置
3. 从全局注册表注销项目
4. 重新加载 Caddy

**重要**：这不会停止容器。需要单独运行 `docker compose down`。

### omd project list

列出所有已注册的项目。

```bash
omd project list
```

**示例输出：**

```
已注册项目:

  • my-api
    路径: /Users/dev/projects/my-api
    域名: my-api.local
    网络: my-api-net
    端口: 5432, 3000

  • my-web
    路径: /Users/dev/projects/my-web
    域名: my-web.local
    网络: my-web-net
    端口: 8080, 8443
```

### omd network list

列出所有 Docker 网络。

```bash
omd network list
```

### omd proxy add

手动添加反向代理规则。

```bash
omd proxy add DOMAIN TARGET
```

**示例：**

```bash
omd proxy add example.local backend:3000
```

### omd proxy remove

删除反向代理规则。

```bash
omd proxy remove DOMAIN
```

### omd proxy list

列出所有代理规则。

```bash
omd proxy list
```

### omd proxy reload

重新加载 Caddy 配置。

```bash
omd proxy reload
```

### omd ports

显示所有网络的端口映射。

```bash
# 所有网络
omd ports

# 特定网络
omd ports show my-api-net
```

## 项目配置

### omd.toml 结构

`omd.toml` 文件位于项目目录中：

```toml
[project]
# 项目名称（用于容器命名）
name = "my-project"

# 此项目的域名
domain = "my-project.local"

[network]
# 此项目的 Docker 网络名称
name = "my-project-net"

[caddy]
# 自定义 Caddy 路由（可选）
routes = {}
```

### 自动路由生成

如果 `[caddy.routes]` 为空或未指定，omd 会自动从您的 `docker-compose.yml` 生成路由：

**docker-compose.yml 示例：**

```yaml
services:
  frontend:
    image: my-frontend
    ports:
      - "8080:80"
    networks:
      - myapp-net

  backend:
    image: my-backend
    ports:
      - "3000:3000"
    networks:
      - myapp-net
```

**生成的路由：**

- `frontend.my-project.local` → `frontend-container:80`
- `backend.my-project.local` → `backend-container:3000`

### 自定义路由

使用自定义路由覆盖自动路由：

```toml
[project]
name = "my-project"
domain = "my-project.local"

[network]
name = "my-project-net"

[caddy.routes]
# 自定义路由: 子域名 -> 容器:端口
api = "my-backend-container:3000"
app = "my-frontend-container:80"
admin = "my-admin-panel:8080"
```

**生成的路由：**

- `api.my-project.local` → `my-backend-container:3000`
- `app.my-project.local` → `my-frontend-container:80`
- `admin.my-project.local` → `my-admin-panel:8080`

## 高级用法

### 端口冲突检测

运行 `omd up` 时，工具会检查全局注册表中的端口冲突：

**场景：**

项目 A 使用端口 5432 用于 PostgreSQL。

您尝试配置项目 B，它也使用端口 5432。

**结果：**

```
✗ 检测到端口冲突:
  端口 5432 已被项目 project-a 使用

由于端口冲突无法继续。
请更新您的 docker-compose.yml 以使用不同的端口。
```

**解决方案：**

更新项目 B 的 `docker-compose.yml`：

```yaml
services:
  postgres:
    image: postgres:15
    ports:
      - "5433:5432"  # 从 5432:5432 更改
```

### 多项目

只要没有端口冲突，您可以同时运行多个项目：

```bash
# 项目 1
cd ~/projects/api-service
omd init
omd up
docker compose up -d
# 通过 https://api-service.local 访问

# 项目 2
cd ~/projects/web-app
omd init
omd up
docker compose up -d
# 通过 https://web-app.local 访问
```

## 故障排除

### 端口已被使用

**错误：**

```
✗ 检测到端口冲突:
  端口 5432 已被项目 another-project 使用
```

**解决方案：**

在 `docker-compose.yml` 中更改主机端口：

```yaml
ports:
  - "5433:5432"  # 使用不同的主机端口
```

### 未找到 omd.toml

**错误：**

```
当前目录未找到 omd.toml。运行 'omd init' 创建一个。
```

**解决方案：**

在项目目录中运行 `omd init`，或导航到正确的目录。

### 未找到 docker-compose.yml

**错误：**

```
当前目录未找到 docker-compose.yml。
请在运行 'omd up' 之前创建 docker-compose.yml。
```

**解决方案：**

在项目目录中创建 `docker-compose.yml` 文件。

## 最佳实践

### 1. 每个项目一个 omd.toml

将 `omd.toml` 保存在项目根目录中，与 `docker-compose.yml` 放在一起。

### 2. 使用显式容器名称

对于类生产环境，显式指定容器名称：

```yaml
services:
  api:
    image: my-api
    container_name: my-project-api
```

### 3. 记录端口分配

在 `docker-compose.yml` 中添加注释：

```yaml
services:
  postgres:
    ports:
      - "5432:5432"  # 标准 PostgreSQL 端口
```

### 4. 一致的命名

在项目名称、网络名称和域名之间使用一致的命名：

```toml
[project]
name = "my-awesome-app"
domain = "my-awesome-app.local"

[network]
name = "my-awesome-app-net"
```

### 5. 启动前检查

在 `docker compose up -d` 之前始终运行 `omd up`：

```bash
omd up              # 配置基础设施
docker compose up -d     # 启动容器
```

### 6. 删除后清理

删除项目时，首先运行 `omd down`：

```bash
cd /path/to/project
docker compose down  # 停止容器
omd down             # 删除配置
cd ..
rm -rf project       # 删除目录
```

### 7. 版本控制

将 `omd.toml` 添加到版本控制，但不要添加注册表：

```gitignore
# .gitignore
# 不要将 registry.json 添加到版本控制
```

但要提交：
```
# 提交 omd.toml
git add omd.toml
```

### 8. 明智使用端口范围

为不同的项目类型分配端口范围：

- 数据库：5000-5999
- API：3000-3999
- Web 服务器：8000-8999

### 9. 网络命名约定

使用描述性网络名称：

- 对于隔离项目：`{项目名}-net`
- 对于共享网络：`{服务类型}-shared`（例如 `microservices-shared`）

### 10. 定期审查注册表

定期查看已注册的项目：

```bash
omd project list
```

使用 `omd down` 删除过时的条目。

---

更多信息，请访问 [GitHub 仓库](https://github.com/your-repo/oh-my-dockers)。
