# upgo API 接口文档

> 前后端通过 **Gateway (Axum :80)** 统一路由，内部服务间通过 gRPC / HTTP 通信。

---

## 目录

1. [Gateway — API 网关](#1-gateway--api-网关)
2. [Auth — 认证服务 (gRPC)](#2-auth--认证服务-grpc)
3. [FRS — 文件存储服务 (REST)](#3-frs--文件存储服务-rest)
4. [Config Manager — 配置管理服务 (REST)](#4-config-manager--配置管理服务-rest)
5. [Account — 账户服务 (gRPC)](#5-account--账户服务-grpc)
6. [Service Mesh](#6-service-mesh)

---

## 1. Gateway — API 网关

**端口**: `:80` (HTTP)
**服务名**: `gateway`

### 路由规则

| 路径前缀 | 目标服务 | 后端端口 |
|----------|----------|----------|
| `/api/auth/*` | Auth | `auth:50052` (gRPC) |
| `/api/files/*` | FRS | `frs:9094` (HTTP) |
| `/api/config/*` | Config Manager | `config-mgr:9095` (HTTP) |
| `/*` | 嵌入式静态文件 | — |

### 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `AUTH_BACKEND` | `http://auth:50052` | Auth 服务地址 |
| `FILES_BACKEND` | `http://frs:9094` | FRS 服务地址 |
| `CONFIG_BACKEND` | `http://config-mgr:9095` | Config Manager 地址 |
| `LISTEN_ADDR` | `0.0.0.0:80` | 监听地址 |
| `LOG_LEVEL` | `info` | 日志级别 |

### 代理行为

- `/api/auth/*` 请求：Gateway 将 `/api/auth/login` 转换为 `POST {AUTH_BACKEND}/auth/login` 透传到 Auth gRPC 端点
- `/api/files/*` 请求：透传到 FRS 的 HTTP 端点
- `/api/config/*` 请求：透传到 Config Manager 的 HTTP 端点
- 静态文件：通过 `rust-embed` 编译进二进制，SPA fallback 到 `index.html`

> **注意**：Gateway 当前做 HTTP→gRPC 透传（需要 Auth 服务后续增加 REST 端点）。
> 目前 `POST /api/auth/login` 等请求会返回 `Bad Gateway`，因为 Auth 只监听 gRPC 端口。

---

## 2. Auth — 认证服务 (gRPC)

**端口**: `50052` (gRPC) / `9090` (Health HTTP)
**服务名**: `auth`
**协议**: gRPC (proto: `contracts/proto/auth.proto`)

### 接口列表

#### Login — 登录

```
rpc Login(LoginRequest) returns (LoginResponse);
```

**LoginRequest**:
| 字段 | 类型 | 说明 |
|------|------|------|
| `email` | string | 用户邮箱 |
| `password` | string | 密码 |
| `platform` | string | 平台: `desktop` / `web` / `mobile` |

**LoginResponse**:
| 字段 | 类型 | 说明 |
|------|------|------|
| `access_token` | string | JWT 访问令牌 |
| `refresh_token` | string | 刷新令牌 |
| `user_id` | string | 用户 UUID |
| `email` | string | 邮箱 |
| `nickname` | string | 昵称 |
| `expires_in` | int32 | 过期时间（秒） |

#### Register — 注册

```
rpc Register(RegisterRequest) returns (RegisterResponse);
```

**RegisterRequest**: `email`, `password`, `nickname`
**RegisterResponse**: `message`（验证码已发送）

#### VerifyEmail — 邮箱验证

```
rpc VerifyEmail(VerifyEmailRequest) returns (VerifyEmailResponse);
```

**VerifyEmailRequest**: `email`, `code`, `platform`
**VerifyEmailResponse**: `access_token`, `refresh_token`, `user_id`, `email`, `nickname`

#### Logout — 登出

```
rpc Logout(LogoutRequest) returns (LogoutResponse);
```

**LogoutRequest**: `session_id`
**LogoutResponse**: `success`

#### LogoutAll — 全设备登出

```
rpc LogoutAll(LogoutAllRequest) returns (LogoutAllResponse);
```

**LogoutAllRequest**: `user_id`
**LogoutAllResponse**: `success`

#### RefreshToken — 刷新令牌

```
rpc RefreshToken(RefreshTokenRequest) returns (RefreshTokenResponse);
```

**RefreshTokenRequest**: `refresh_token`
**RefreshTokenResponse**: `access_token`, `refresh_token`, `expires_in`

#### ChangePassword — 修改密码

```
rpc ChangePassword(ChangePasswordRequest) returns (ChangePasswordResponse);
```

**ChangePasswordRequest**: `user_id`, `old_password`, `new_password`
**ChangePasswordResponse**: `success`
> 通过 gRPC metadata `x-session-id` 验证当前会话

#### ChangeEmail — 申请更换邮箱

```
rpc ChangeEmail(ChangeEmailRequest) returns (ChangeEmailResponse);
```

**ChangeEmailRequest**: `user_id`, `new_email`
**ChangeEmailResponse**: `message`

#### ConfirmEmailChange — 确认更换邮箱

```
rpc ConfirmEmailChange(ConfirmEmailChangeRequest) returns (ConfirmEmailChangeResponse);
```

**ConfirmEmailChangeRequest**: `user_id`, `code`
**ConfirmEmailChangeResponse**: `success`, `new_email`

#### DeleteAccount — 申请注销账户

```
rpc DeleteAccount(DeleteAccountRequest) returns (DeleteAccountResponse);
```

**DeleteAccountRequest**: `user_id`
**DeleteAccountResponse**: `message`（验证码已发送）

#### ConfirmDeleteAccount — 确认注销

```
rpc ConfirmDeleteAccount(ConfirmDeleteAccountRequest) returns (ConfirmDeleteAccountResponse);
```

**ConfirmDeleteAccountRequest**: `user_id`, `code`
**ConfirmDeleteAccountResponse**: `success`, `deleted_at`

#### CancelDeleteAccount — 取消注销

```
rpc CancelDeleteAccount(CancelDeleteAccountRequest) returns (CancelDeleteAccountResponse);
```

**CancelDeleteAccountRequest**: `user_id`
**CancelDeleteAccountResponse**: `success`

#### GetSessions — 获取会话列表

```
rpc GetSessions(GetSessionsRequest) returns (GetSessionsResponse);
```

**GetSessionsRequest**: `user_id`
**GetSessionsResponse**: `sessions[]`
> `SessionInfo`: `session_id`, `platform`, `created_at`, `last_active_at`, `is_current`

### gRPC 错误码

| 错误场景 | gRPC Code | HTTP 等效 |
|----------|-----------|-----------|
| 凭据无效 | `Unauthenticated` | 401 |
| 弱密码 | `InvalidArgument` | 400 |
| 邮箱已存在 | `AlreadyExists` | 409 |
| 账户已冻结/已删除 | `PermissionDenied` | 403 |
| 账户不存在 | `NotFound` | 404 |
| 令牌过期/无效 | `Unauthenticated` | 401 |
| 验证码错误/过期 | `InvalidArgument` | 400 |

### 健康检查

```
GET /health → "OK"
```
端口: `9090` (HTTP)

---

## 3. FRS — 文件存储服务 (REST)

**端口**: `9094` (HTTP)
**服务名**: `frs`
**协议**: REST (JSON)

### 接口列表

#### Upload — 上传文件

```
POST /api/files/upload?key=<key>&content_type=<mime>
Body: <file bytes>
```

**Query 参数**:
| 参数 | 类型 | 必填 | 默认 | 说明 |
|------|------|------|------|------|
| `key` | string | 否 | UUID v7 | 文件存储路径 |
| `content_type` | string | 否 | — | MIME 类型 |

**Response 200**:
```json
{ "key": "uuid-key", "etag": "abc123", "size": 1024 }
```

#### Download — 下载文件

```
GET /api/files/download/{key}
```

**Path 参数**: `key` — 文件存储路径

**Response 200**: 文件内容流（原始字节 + Content-Type / Content-Length 头）

**Response 404**:
```json
{ "error": "File not found" }
```

#### Delete — 删除文件

```
DELETE /api/files/delete/{key}
```

**Response 200**:
```json
{ "status": "deleted", "key": "xxx" }
```

#### Info — 文件元信息

```
GET /api/files/info/{key}
```

**Response 200**:
```json
{
  "key": "uuid-key",
  "size": 1024,
  "content_type": "image/png",
  "etag": "abc123",
  "last_modified": "2026-06-18T10:00:00Z"
}
```

#### List — 列出文件

```
GET /api/files/list?prefix=<prefix>&max_keys=<n>
```

**Query 参数**:
| 参数 | 类型 | 必填 | 默认 | 说明 |
|------|------|------|------|------|
| `prefix` | string | 否 | — | 路径前缀过滤 |
| `max_keys` | int | 否 | 1000 | 最大返回数 |

**Response 200**:
```json
{
  "files": [{ "key": "xxx", "size": 1024, "etag": "...", "last_modified": "..." }],
  "is_truncated": false
}
```

#### Presigned Upload URL — 生成预签名上传 URL

```
POST /api/files/presigned/upload
Content-Type: application/json

{ "key": "uuid-key (optional)", "expires_secs": 3600 }
```

**Response 200**:
```json
{ "url": "https://minio:9000/...?X-Amz-Signature=...", "key": "uuid-key", "expires_in": 3600 }
```

#### Presigned Download URL — 生成预签名下载 URL

```
GET /api/files/presigned/download/{key}?expires_secs=3600
```

**Response 200**:
```json
{ "url": "https://minio:9000/...", "key": "xxx", "expires_in": 3600 }
```

### 配置

| 环境变量 | 默认值 | 说明 |
|----------|--------|------|
| `HTTP_ADDR` | `0.0.0.0:9094` | HTTP 监听地址 |
| `S3_ENDPOINT` | `http://minio:9000` | MinIO S3 端点 |
| `S3_ACCESS_KEY` | `minioadmin` | 凭证 |
| `S3_SECRET_KEY` | `minioadmin` | 凭证 |
| `S3_BUCKET` | `upgo-files` | 存储桶 |

---

## 4. Config Manager — 配置管理服务 (REST)

**端口**: `9095` (HTTP)
**服务名**: `config-mgr`
**协议**: REST (JSON)
**底座**: RNacos (`rnacos:8848`)

### 接口列表

#### Get Config — 获取配置

```
GET /api/config/get?data_id=<id>&group=<group>
```

**Query 参数**:
| 参数 | 类型 | 必填 | 默认 | 说明 |
|------|------|------|------|------|
| `data_id` | string | 是 | — | 配置 ID |
| `group` | string | 否 | `DEFAULT_GROUP` | 配置分组 |

**Response 200**:
```json
{
  "data_id": "my-config",
  "group": "DEFAULT_GROUP",
  "content": "{ \"key\": \"value\" }",
  "cached": true
}
```
> `cached: true` 表示来自本地缓存，`false` 表示实时拉取

#### Publish Config — 发布配置

```
POST /api/config/publish
Content-Type: application/json

{ "data_id": "my-config", "group": "DEFAULT_GROUP", "content": "...", "content_type": "json" }
```

**Request Body**:
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `data_id` | string | 是 | 配置 ID |
| `group` | string | 否 | 分组 |
| `content` | string | 是 | 配置内容 |
| `content_type` | string | 否 | 类型标识 |

**Response 200**:
```json
{ "success": true, "data_id": "my-config", "group": "DEFAULT_GROUP" }
```

#### Delete Config — 删除配置

```
DELETE /api/config/delete?data_id=<id>&group=<group>
```

**Response 200**:
```json
{ "success": true, "data_id": "my-config", "group": "DEFAULT_GROUP" }
```

#### List Configs — 按组列出配置

```
GET /api/config/list/{group}
```

**Path 参数**: `group` — 配置分组名

**Response 200**:
```json
{
  "configs": [
    { "data_id": "my-config", "group": "DEFAULT_GROUP", "version": 1 }
  ]
}
```

#### Watch Configs — 监听配置变更（长轮询推送）

```
POST /api/config/watch
Content-Type: application/json

{
  "data_ids": [{ "data_id": "my-config", "group": "DEFAULT_GROUP" }],
  "timeout_ms": 30000
}
```

**Request Body**:
| 字段 | 类型 | 说明 |
|------|------|------|
| `data_ids[]` | array | 要监听的配置列表 |
| `data_ids[].data_id` | string | 配置 ID |
| `data_ids[].group` | string | 分组 |
| `timeout_ms` | int | 长轮询超时（默认 30s） |

**Response 200（有变更）**:
```json
{
  "changed": [{ "data_id": "my-config", "group": "DEFAULT_GROUP" }]
}
```

**Response 200（超时，无变更）**:
```json
{ "changed": [] }
```

#### Preload Cache — 预加载配置到本地缓存

```
POST /api/config/cache/preload
Content-Type: application/json

{
  "configs": [{ "data_id": "my-config", "group": "DEFAULT_GROUP" }]
}
```

**Response 200**:
```json
{ "success": true, "preloaded": 5 }
```

### 配置

| 环境变量 | 默认值 | 说明 |
|----------|--------|------|
| `HTTP_ADDR` | `0.0.0.0:9095` | 监听地址 |
| `RNACOS_ADDR` | `http://rnacos:8848` | RNacos 底座地址 |
| `RNACOS_NAMESPACE` | `public` | Nacos 命名空间 |

---

## 5. Account — 账户服务 (gRPC)

**端口**: `50051` (gRPC)
**服务名**: `account`
**状态**: 🔧 开发中（gRPC server 当前为 stub 实现）

### Proto 定义

| 文件 | 内容 |
|------|------|
| `contracts/proto/user.proto` | 用户账户接口 |
| `contracts/proto/fund.proto` | 资金账户接口 |
| `contracts/proto/position.proto` | 持仓接口 |
| `contracts/proto/risk.proto` | 风控接口 |

> 具体 gRPC 方法定义请参考 proto 文件：
> `contracts/proto/*.proto`

---

## 6. Service Mesh

### 内部服务拓扑

```
                          ┌─────────────────┐
                          │   Gateway :80    │
                          │  (API 网关 / 静态文件) │
                          └────────┬────────┘
                 ┌─────────────────┼────────────────────┐
                 ▼                 ▼                    ▼
        ┌────────────────┐ ┌──────────────┐ ┌──────────────────┐
        │ Auth :50052    │ │ FRS :9094    │ │ Config-Mgr :9095 │
        │ (gRPC 认证服务) │ │ (REST 文件存储)│ │ (REST 配置管理)   │
        └───────┬────────┘ └──────┬───────┘ └────────┬─────────┘
                │                 │                   │
                ▼                 ▼                   ▼
        ┌────────────┐  ┌──────────────┐  ┌────────────────┐
        │ PostgreSQL │  │ MinIO :9000  │  │ RNacos :8848   │
        │  :5432     │  │ (S3 存储)    │  │ (配置中心)      │
        └────────────┘  └──────────────┘  └────────────────┘
```

### 可观测性（OpenTelemetry）

```
                  OTLP gRPC (:4317)
[Auth] ──────────┐
[Gateway] ───────┤──→ [SigNoz Collector]
[FRS] ───────────┤       │
[Config-Mgr] ────┘       ├── ClickHouse
                         └── Logging (debug)
```

### 基础设施

| 组件 | 端口 | 说明 |
|------|------|------|
| PostgreSQL | 5432 | 主数据库 |
| NATS | 4222 | 消息队列 |
| Redis | 6379 | 缓存 |
| MinIO | 9000 / 9001 | 对象存储 (S3) |
| RNacos | 8848 / 9848 | 配置中心 |
| Mailpit | 1025 / 8025 | 邮件测试 (SMTP) |
| SigNoz Collector | 4317 / 4318 | OTLP 收集器 |
| ClickHouse | 8123 / 9000 | 时序数据库 |
