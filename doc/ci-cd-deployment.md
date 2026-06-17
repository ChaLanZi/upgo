# upgo CI/CD 部署操作流程

> 完整流程：代码编译验证 → 测试 → 构建 Docker 镜像 → 部署到 Minikube k8s 集群

---

## 流程概览

```mermaid
flowchart LR
    A[1. 前置检查] --> B[2. Dagger CI 流水线]
    B --> C[3. 启动 Minikube]
    C --> D[4. 部署基础设施]
    D --> E[5. 构建服务镜像]
    E --> F[6. 部署业务服务]
    F --> G[7. 验证部署]
    G -.-> H[问题修复]
    H -.-> F
    style A fill:#f0f0f0
    style B fill:#4a9eff,color:#fff
    style E fill:#f0f0f0
    style F fill:#4a9eff,color:#fff
    style H fill:#f44336,color:#fff
```

---

## 步骤

### 1. 前置环境检查

```bash
# 检查工具链
rustc --version          # 需 1.85+
minikube version         # 需 latest
kubectl version --client # 需 latest
docker --version         # 需 latest
just --version           # 需 latest
dagger version           # 需 latest
cargo nextest --version  # 需 latest

# 检查 Docker 运行状态
docker info
```

### 2. Dagger CI 流水线（编译验证 + 测试）

**目标：** 验证代码编译通过，单元测试和集成测试全部通过。

```bash
cd upgo

# 方式 A：Shell 模式（推荐本地开发，可运行 Docker 集成测试）
just dagger-ci
# 等价于：dagger run sh -c 'cargo check && RUSTFLAGS="--cfg docker_tests" cargo nextest run'

# 方式 B：Go SDK 模式（隔离容器环境，编译验证 + nextest）
just dagger-ci-go
# 等价于：dagger run go run ./ci/ ci
```

> ✅ 所有测试通过后，继续后续部署步骤。

### 3. 启动 Minikube 集群

```bash
# 启动 Minikube（Docker 驱动）
minikube start --driver=docker --cpus=4 --memory=4096

# 验证集群状态
kubectl cluster-info
kubectl get nodes
```

### 4. 部署基础设施服务

基础设施包括：PostgreSQL、NATS、Redis、RNacos、SigNoz、Mailpit。

```bash
cd upgo

# 部署所有基础设施到 k8s
kubectl apply -k k8s/overlays/dev

# 等待所有 Deployment 就绪
kubectl wait --for=condition=Available deployments --all --timeout=180s

# 验证状态
kubectl get pods
```

**预期状态：**
| Pod | 状态 |
|-----|------|
| postgres-* | Running |
| redis-* | Running |
| rnacos-* | Running |
| signoz-otel-collector-* | Running |
| mailpit-* | Running |
| auth-* | Pending（镜像未构建）|
| nats-* | Running（StatefulSet）|

> NATS 和 ClickHouse 为 StatefulSet，需单独验证：`kubectl get statefulsets`

### 5. 构建业务服务镜像

#### 5.1 准备 Docker 镜像加速器（中国网络必需）

在中国网络环境下，Docker Hub 不可达。需要配置镜像加速器：

```bash
# 配置 DaoCloud 镜像加速器
cat > ~/.docker/daemon.json << 'EOF'
{
  "registry-mirrors": ["https://docker.m.daocloud.io"]
}
EOF

# 重启 Docker Desktop
osascript -e 'quit app "Docker"'; sleep 2; open -a Docker
```

验证镜像加速器生效：
```bash
docker info | grep -A2 "Registry Mirrors"
# 输出：https://docker.m.daocloud.io/
```

#### 5.2 构建 Auth 服务镜像

使用多阶段 Dockerfile 确保二进制与目标 Linux 架构一致：

```bash
cd upgo

# 确保 .dockerignore 已配置（排除 target/ .git/ 等）
cat > .dockerignore << 'EOF'
target/
.git/
node_modules/
frontend/
ci/
doc/
k8s/
EOF

# 多阶段构建（编译在 rust:slim 容器中，产出 Linux 二进制）
docker build -t upgo-auth:latest -f Dockerfile.auth .
# 首次构建约需 10 分钟（Rust 编译耗时），后续有缓存会快很多
```

#### 5.3 加载镜像到 Minikube

```bash
# 方式一：通过 docker save + load 加载（推荐，稳定可靠）
docker save upgo-auth:latest -o /tmp/upgo-auth.tar
docker cp /tmp/upgo-auth.tar minikube:/upgo-auth.tar
docker exec minikube docker load -i /upgo-auth.tar

# 方式二（备选）：minikube image load（可能因网络问题超时）
minikube image load upgo-auth:latest
```

#### 5.4 Account 服务

```
项目状态：Account 服务的 gRPC server 当前为 stub 实现，
尚未完全就绪，跳过部署。后续开发完成后：
cargo build --release -p account
docker build -t upgo-account:latest -f Dockerfile.account .
```

#### 5.5 Frontend（WASM 前端）

```
项目状态：Frontend-auth 为 WASM 前端，
需要 Nginx 或类似静态文件服务容器化，暂未就绪。
```

### 6. 部署业务服务到 k8s

```bash
cd upgo

# 部署 Auth 服务（使用 kustomize）
kubectl apply -k k8s/overlays/dev

# 重启以使新镜像生效
kubectl rollout restart deployment auth

# 等待就绪
kubectl rollout status deployment auth --timeout=120s
```

### 7. 验证部署

```bash
# 查看所有 Pod 状态
kubectl get pods

# 查看服务
kubectl get services

# 测试 Auth 服务健康检查
kubectl port-forward svc/auth 9090:9090 &
curl -s http://localhost:9090/health
# 预期输出：OK

# 查看服务日志
kubectl logs -l app=auth

# 确认 gRPC 端口监听
kubectl logs -l app=auth | grep "gRPC server listening"
```

---

## 常见问题与修复（实际验证）

### 🔴 Docker Hub 不可达（中国网络）

**现象：** Pod 处于 `ImagePullBackOff` / `ErrImagePull`，`kubectl describe pod` 显示
`Failed to pull image "xxx"：context deadline exceeded`

**修复步骤：**

```bash
# 1. 从可访问的主机 Docker pull 镜像
docker pull axllent/mailpit:latest

# 2. 加载到 Minikube 容器
docker save axllent/mailpit:latest -o /tmp/mailpit.tar
docker cp /tmp/mailpit.tar minikube:/mailpit.tar
docker exec minikube docker load -i /mailpit.tar

# 3. 设置 imagePullPolicy: IfNotPresent 避免 kubelet 再拉取
# 编辑 k8s/base/<service>-deployment.yaml，在 containers 下添加：
#   imagePullPolicy: IfNotPresent

# 4. 重启 Deployment
kubectl rollout restart deployment <service>
```

### 🔴 exec format error（架构不匹配）

**现象：** Pod 状态 `CrashLoopBackOff`，日志显示 `exec /usr/local/bin/auth: exec format error`

**原因：** 在 macOS（Darwin）上编译的二进制无法在 Linux 容器中运行。

**修复：** 使用 Docker 多阶段构建（`Dockerfile.auth`）在 Linux 容器内编译，确保二进制格式一致。

### 🔴 健康检查探针不匹配

**现象：** Pod 反复重启，`Liveness probe failed: HTTP probe failed with statuscode: 404`

**修复：** 某些服务（如 Mailpit）没有 `/health` 端点，改用 TCP 探针：

```yaml
# 从 HTTP GET 改为 TCP Socket
livenessProbe:
  tcpSocket:
    port: 8025
  initialDelaySeconds: 10
  periodSeconds: 15
```

### 🔴 数据库不存在

**现象：** Pod 日志显示 `database "upgo_auth" does not exist`

**修复：** 在 Postgres Pod 中创建数据库：

```bash
kubectl exec <postgres-pod> -- psql -U postgres -c "CREATE DATABASE upgo_auth;"
```

### 🔴 镜像构建超时

**现象：** `docker build` 长时间卡住后超时

**修复：** 创建 `.dockerignore` 缩小构建上下文；首次构建约需 10 分钟
（Rust 编译），之后有层缓存会快很多。

---

## 最终部署状态

| 组件 | 端口 | 状态 |
|------|------|------|
| Auth Service (gRPC) | 50052 | ✅ Running |
| Auth Service (Health) | 9090 | ✅ OK |
| PostgreSQL | 5432 | ✅ Running |
| NATS | 4222 | ✅ Running |
| Redis | 6379 | ✅ Running |
| Mailpit | 1025/8025 | ✅ Running |
| RNacos | 8848 | ✅ Running |
| SigNoz | 4317 | ✅ Running |

## 清理

```bash
# 仅停止 Minikube（保留数据）
minikube stop

# 完全重置
minikube delete && minikube start --driver=docker --cpus=4 --memory=4096

# 使用 Justfile
just k8s-down
```

## 问题排查命令速查

```bash
# 查看 Pod 详情
kubectl describe pod <pod-name>

# 查看日志
kubectl logs <pod-name>

# 端口转发测试
kubectl port-forward svc/auth 9090:9090 &

# 进入 Pod 调试
kubectl exec -it <pod-name> -- /bin/sh

# 重启 Deployment
kubectl rollout restart deployment <name>

# 删除停滞 Pod
kubectl delete pod -l app=<name> --force
```
