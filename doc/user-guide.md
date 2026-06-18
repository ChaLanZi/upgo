# upgo 用户使用指南

## 入口地址

应用运行在 Minikube Kubernetes 集群中，通过 **Gateway** 统一对外提供服务。

### 获取入口 IP 和端口

```bash
# 查看 Gateway 的 Service 信息
kubectl get svc -n upgo frontend

# 或者通过 Minikube 获取可直接访问的 URL
minikube service frontend -n upgo --url
```

### 端口映射

| 服务 | 内部端口 | 对外方式 | 说明 |
|------|----------|----------|------|
| **Gateway (前端)** | `:80` | `minikube service frontend` | Web 应用入口 |
| **Auth 健康检查** | `:9090` | `kubectl port-forward` | 健康检查 |
| **MinIO Console** | `:9001` | `kubectl port-forward` | 文件管理后台 |
| **RNacos Console** | `:8848` | `kubectl port-forward` | 配置管理后台 |
| **Mailpit** | `:8025` | `kubectl port-forward` | 邮件测试界面 |

### 快速访问命令

```bash
# 一键获取所有服务 URL
minikube service list -n upgo

# 打开前端网页
minikube service frontend -n upgo

# 端口转发：访问 MinIO 管理后台 (用户: minioadmin / minioadmin)
kubectl port-forward -n upgo svc/minio 9001:9001
# 访问 http://localhost:9001

# 端口转发：访问 RNacos 配置中心
kubectl port-forward -n upgo svc/rnacos 8848:8848
# 访问 http://localhost:8848

# 端口转发：访问 Mailpit 邮件测试
kubectl port-forward -n upgo svc/mailpit 8025:8025
# 访问 http://localhost:8025

# 端口转发：访问 Auth 健康检查
kubectl port-forward -n upgo svc/auth 9090:9090
# 访问 http://localhost:9090/health
```

## 功能使用

### 用户认证

Gateway 路由到 Auth 服务（gRPC），当前前端通过 Gateway 的 `/api/auth/*` 路径转发。

**登录/注册**：在首页输入邮箱和密码即可注册/登录。

### 文件管理

FRS 服务通过 `/api/files/*` 路径访问，由 Gateway 代理到 `frs:9094`。

| 功能 | 说明 |
|------|------|
| 上传文件 | 支持指定 key 或自动生成 UUID |
| 下载文件 | 通过 key 直接下载 |
| 删除文件 | 通过 key 删除 |
| 文件列表 | 按前缀过滤列出 |
| 预签名 URL | 生成临时直传/直下链接 |

前端页面内置文件管理 UI，登录后显示文件管理面板。

### 配置管理

Config Manager 通过 `/api/config/*` 路径访问，由 Gateway 代理到 `config-mgr:9095`，后端存储使用 RNacos。

| 功能 | 说明 |
|------|------|
| 获取配置 | GET `/api/config/get?data_id=xxx` |
| 发布配置 | POST `/api/config/publish` |
| 监听变更 | POST `/api/config/watch`（长轮询推送） |
| 预加载缓存 | POST `/api/config/cache/preload` |

### 可观测性 (SigNoz)

所有服务通过 OTLP gRPC 协议向 SigNoz Collector 上报追踪和日志数据。

```bash
# 访问 SigNoz 界面（需要部署 SigNoz Query Service）
# 确认 OTel Collector 运行状态
kubectl get pods -n upgo -l app=signoz-otel-collector
```

## 一键命令

```bash
# 完整部署
just deploy-all

# 查看部署状态和所有 Pod
kubectl get pods -n upgo

# 查看服务端点
kubectl get endpoints -n upgo

# 查看日志
just k8s-logs

# 验证健康检查
just verify
```

## 架构概览

```
用户 → [Gateway :80] → [Auth / FRS / Config-Mgr / 静态文件]
         ↓
[MinIO :9000] — 文件存储
[RNacos :8848] — 配置中心
[PostgreSQL :5432] — 主数据库
[NATS :4222] — 消息队列
[Redis :6379] — 缓存
```

所有数据通过 OpenTelemetry `:4317` 上报到 SigNoz Collector → ClickHouse 持久化。
