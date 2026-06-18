# ──────────────────────────────────────────────────────────
# upgo 本地开发环境一键命令
# 依赖：cargo install just
# 需预装：minikube, kubectl, docker, dagger
# ──────────────────────────────────────────────────────────

dotenv-load:
    test -f .env && set -a && source .env 2>/dev/null; true

# ══════════════════════════════════════════════════════════
# K8s 集群管理
# ══════════════════════════════════════════════════════════

# 启动 Minikube 并部署基础设施
k8s-up:
    @echo "=== Checking Minikube status ==="
    minikube status || minikube start --driver=docker --cpus=2 --memory=4096
    @echo "=== Creating namespace ==="
    kubectl create namespace upgo --dry-run=client -o yaml | kubectl apply -f -
    @echo "=== Deploying k8s infrastructure ==="
    kubectl apply -k k8s/overlays/dev
    @echo "=== Waiting for deployments ready ==="
    kubectl wait --for=condition=Available deployments --all -n upgo --timeout=120s
    @echo "=== Creating databases ==="
    -kubectl exec -n upgo deploy/postgres -- psql -U postgres -c "CREATE DATABASE upgo_auth;" 2>/dev/null
    -kubectl exec -n upgo deploy/postgres -- psql -U postgres -c "CREATE DATABASE upgo_account;" 2>/dev/null
    @echo "=== Exposing services ==="
    minikube service postgres --url -n upgo
    minikube service nats --url -n upgo

# 暂停 Minikube（保留数据）
k8s-down:
    @echo "=== Stopping Minikube ==="
    minikube stop

# 重置 Minikube（删除并重建）
k8s-reset:
    @echo "=== Resetting Minikube ==="
    minikube delete
    minikube start --driver=docker --cpus=2 --memory=4096
    kubectl create namespace upgo --dry-run=client -o yaml | kubectl apply -f -
    just infra-deploy
    just infra-db

# 查看基础设施日志
k8s-logs:
    @echo "=== Streaming all pod logs ==="
    kubectl logs -n upgo -l 'app in (postgres, nats, redis, signoz-otel-collector, clickhouse, rnacos)' --tail=50 -f

# ══════════════════════════════════════════════════════════
# 基础设施部署
# ══════════════════════════════════════════════════════════

# 部署基础设施到 k8s
infra-deploy:
    @echo "=== Creating namespace ==="
    kubectl create namespace upgo --dry-run=client -o yaml | kubectl apply -f -
    @echo "=== Deploying infrastructure ==="
    kubectl apply -k k8s/overlays/dev
    kubectl wait --for=condition=Available deployments --all -n upgo --timeout=120s

# 创建服务所需的数据库
infra-db:
    @echo "=== Creating databases ==="
    -kubectl exec -n upgo deploy/postgres -- psql -U postgres -c "CREATE DATABASE upgo_auth;" 2>/dev/null
    -kubectl exec -n upgo deploy/postgres -- psql -U postgres -c "CREATE DATABASE upgo_account;" 2>/dev/null

# ══════════════════════════════════════════════════════════
# 基础设施镜像缓存（中国网络必需）
# ══════════════════════════════════════════════════════════

# 预拉取并缓存基础设施镜像到 Minikube（避免 ImagePullBackOff）
infra-cache:
    @echo "=== Pre-caching infrastructure images into Minikube ==="
    @declare -a IMAGES=( \
      "postgres:16" \
      "redis:7" \
      "nats:2.10" \
      "axllent/mailpit:latest" \
      "qingpan/rnacos:latest" \
      "clickhouse/clickhouse-server:24.3" \
      "signoz/signoz-otel-collector:0.88.17" \
      "quay.io/minio/minio:latest" \
    ); \
    for img in "$${IMAGES[@]}"; do \
      echo "Pulling $$img..."; \
      docker pull "$$img" || echo "⚠️  Pull failed for $$img"; \
    done; \
    for img in "$${IMAGES[@]}"; do \
      name=$$(echo "$$img" | tr '/' '_' | tr ':' '_'); \
      echo "Loading $$img into Minikube..."; \
      docker save "$$img" -o "/tmp/infra_$${name}.tar" 2>/dev/null && \
      docker cp "/tmp/infra_$${name}.tar" minikube:"/infra_$${name}.tar" 2>/dev/null && \
      docker exec minikube docker load -i "/infra_$${name}.tar" 2>/dev/null || \
      echo "⚠️  Failed to load $$img"; \
    done
    @echo "=== Cache complete ==="
    minikube image ls

# 查看已缓存的镜像
infra-cache-list:
    @echo "=== Cached images in Minikube ==="
    minikube image ls

# ══════════════════════════════════════════════════════════
# 业务服务构建与部署
# ══════════════════════════════════════════════════════════

# 构建 Auth 服务镜像（多阶段构建，保证 Linux 二进制）
build-auth:
    @echo "=== Building auth service image ==="
    docker build -t upgo-auth:latest -f Dockerfile.auth .
    @echo "=== Image built: ==="
    docker images upgo-auth

# 加载 Auth 镜像到 Minikube
load-auth:
    @echo "=== Loading auth image into Minikube ==="
    docker save upgo-auth:latest -o /tmp/upgo-auth.tar
    docker cp /tmp/upgo-auth.tar minikube:/upgo-auth.tar
    docker exec minikube docker load -i /upgo-auth.tar
    @echo "=== Verify ==="
    docker exec minikube docker images upgo-auth

# 构建 + 加载 Auth 镜像（一步完成）
build-auth-full: build-auth load-auth

# 部署 Auth 服务（重启使用新镜像）
deploy-auth:
    @echo "=== Deploying auth service ==="
    kubectl apply -k k8s/overlays/dev
    kubectl rollout restart deployment auth -n upgo
    kubectl rollout status deployment auth -n upgo --timeout=120s

# 构建 + 加载 + 部署 Auth 服务（全自动）
deploy-auth-full: build-auth-full deploy-auth

# 构建 FRS 服务镜像（多阶段构建）
build-frs:
    @echo "=== Building frs service image ==="
    docker build -t upgo-frs:latest -f Dockerfile.frs .
    @echo "=== Image built: ==="
    docker images upgo-frs

# 加载 FRS 镜像到 Minikube
load-frs:
    @echo "=== Loading frs image into Minikube ==="
    docker save upgo-frs:latest -o /tmp/upgo-frs.tar
    docker cp /tmp/upgo-frs.tar minikube:/upgo-frs.tar
    docker exec minikube docker load -i /upgo-frs.tar
    @echo "=== Verify ==="
    docker exec minikube docker images upgo-frs

# 构建 + 加载 FRS 镜像（一步完成）
build-frs-full: build-frs load-frs

# 部署 FRS 服务（重启使用新镜像）
deploy-frs:
    @echo "=== Deploying frs service ==="
    kubectl apply -k k8s/overlays/dev
    kubectl rollout restart deployment frs -n upgo
    kubectl rollout status deployment frs -n upgo --timeout=120s

# 构建 + 加载 + 部署 FRS 服务（全自动）
deploy-frs-full: build-frs-full deploy-frs

# ══════════════════════════════════════════════════════════
# 测试
# ══════════════════════════════════════════════════════════

# 运行单元测试（无需基础设施）
test-unit:
    @echo "=== Running unit tests ==="
    cargo test --lib -p auth
    cargo test --lib -p account

# 运行全部测试（集成测试需 Docker）
test-all:
    @echo "=== Running all tests ==="
    RUSTFLAGS='--cfg docker_tests' cargo nextest run

# ══════════════════════════════════════════════════════════
# Dagger CI/CD 流水线
# ══════════════════════════════════════════════════════════

# Shell 模式（本地开发，推荐，可运行 Docker 集成测试）
dagger-ci:
    @echo "=== Running Dagger CI pipeline (shell) ==="
    dagger run sh -c 'cd {{ justfile_directory() }} && cargo check && RUSTFLAGS="--cfg docker_tests" cargo nextest run'

# Go SDK 模式（隔离容器环境，编译验证 + nextest）
dagger-ci-go:
    @echo "=== Running Dagger CI pipeline (Go SDK) ==="
    dagger run go run ./ci/ ci

# ══════════════════════════════════════════════════════════
# 一键流程
# ══════════════════════════════════════════════════════════

# 完整部署流程：缓存镜像 → 部署基础设施 → 构建服务 → 部署服务
deploy-all: infra-cache k8s-up build-auth-full deploy-auth build-frs-full deploy-frs
    @echo "=== All services deployed ==="
    kubectl get pods -n upgo

# 完整 CI 流程：启动集群 → 运行测试 → 清理
ci: k8s-up
    @echo "=== Running CI pipeline (shell) ==="
    dagger run sh -c 'cd {{ justfile_directory() }} && cargo check && RUSTFLAGS="--cfg docker_tests" cargo nextest run'
    @echo "=== Cleaning up ==="
    minikube stop

# 完整部署后验证
verify:
    @echo "=== Pod Status ==="
    kubectl get pods -n upgo
    @echo ""
    @echo "=== Service Endpoints ==="
    kubectl get endpoints -n upgo
    @echo ""
    @echo "=== Auth Health Check ==="
    -kubectl port-forward -n upgo svc/auth 9090:9090 &
    @sleep 2
    -curl -s http://localhost:9090/health && echo "" || echo "⚠️  Auth health check failed"
    @-kill %1 2>/dev/null
    @echo "=== FRS Health Check ==="
    -kubectl port-forward -n upgo svc/frs 9094:9094 &
    @sleep 2
    -curl -s http://localhost:9094/api/files/list && echo "" || echo "⚠️  FRS health check failed"
    @-kill %2 2>/dev/null
