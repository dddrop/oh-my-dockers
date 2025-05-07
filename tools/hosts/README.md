# Hosts Management Tool

自动管理 `/etc/hosts` 文件的工具，为 oh-my-dockers 项目添加域名解析。

## 功能特性

- ✅ **安全标记**：使用特殊注释标记管理的条目，不影响其他配置
- ✅ **通配符支持**：自动添加 `domain` 和 `*.domain` 
- ✅ **交互确认**：显示变更 diff，等待用户确认
- ✅ **自动备份**：修改前创建 `/etc/hosts.backup`
- ✅ **原子操作**：通过临时文件确保安全写入

## 使用方法

### 添加项目域名

```bash
task hosts:add PROJECT=daily
```

**输出示例**：
```
Reading /etc/hosts...

Changes to be applied:
  + # oh-my-dockers: daily
  + 127.0.0.1 daily.local n8n.daily.local chroma.daily.local surrealdb.daily.local
  + # oh-my-dockers: end

Apply these changes to /etc/hosts? [y/N]: y

Updating /etc/hosts...
ℹ Created backup: /etc/hosts.backup
[sudo] password for user: 
✓ Hosts file updated successfully

Added domains:
  • daily.local
  • n8n.daily.local
  • chroma.daily.local
  • surrealdb.daily.local
```

### 移除项目域名

```bash
task hosts:remove PROJECT=daily
```

### 列出管理的条目

```bash
task hosts:list
```

**输出**：
```
oh-my-dockers managed entries:

  • daily
  • example
```

### 清理所有条目

```bash
task hosts:clean
```

## 标记格式

hosts 工具使用以下格式标记管理的条目：

```
# oh-my-dockers: daily
127.0.0.1 daily.local n8n.daily.local chroma.daily.local surrealdb.daily.local
# oh-my-dockers: end
```

- 开始标记：`# oh-my-dockers: <project-name>`
- 结束标记：`# oh-my-dockers: end`
- 内容：`127.0.0.1 <domain> <subdomain1>.<domain> <subdomain2>.<domain> ...`

**注意**：/etc/hosts 不支持通配符，所以工具会列出所有实际需要的子域名（基于项目配置中启用的 HTTP 服务）。

## 安全机制

1. **备份**：每次修改前自动创建 `/etc/hosts.backup`
2. **验证**：写入前验证文件格式
3. **原子性**：使用临时文件，成功后才替换
4. **权限**：使用 sudo，需要用户密码确认

## 集成到工作流

hosts 工具已集成到 `task setup` 命令中，会在初始化时自动询问是否添加域名。

```bash
# 自动处理（推荐）
task setup  # 会自动运行 task hosts:add PROJECT=daily

# 手动管理
task hosts:add PROJECT=myapp
task hosts:remove PROJECT=myapp
```

## 故障排除

### 权限被拒绝

确保你的用户有 sudo 权限：
```bash
sudo -v
```

### 域名未生效

修改后可能需要清除 DNS 缓存：
```bash
# macOS
sudo dscacheutil -flushcache
sudo killall -HUP mDNSResponder

# Linux
sudo systemctl restart systemd-resolved
```

### 恢复备份

如果出现问题，可以从备份恢复：
```bash
sudo cp /etc/hosts.backup /etc/hosts
```

