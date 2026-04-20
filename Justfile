# ──────────────────────────────────────────────
# upgo 本地开发环境一键命令
# 依赖：cargo install just
# 需预装：minikube, kubectl, dagger
# ──────────────────────────────────────────────

dotenv-load:
    test -f .env && set -a && source .env 2>/dev/null; true

# 启动 Minikube 并部署基础设施
k8s-up:
    @echo "=== Checking Minikube status ==="
    minikube status || minikube start --driver=docker --cpus=2 --memory=2048
    @echo "=== Deploying k8s infrastructure ==="
    kubectl apply -k k8s/overlays/dev
    @echo "=== Waiting for deployments ready ==="
    kubectl wait --for=condition=Available deployments --all --timeout=120s
    @echo "=== Exposing services ==="
    minikube service postgres --url
    minikube service nats --url

# 运行全部测试（确保 k8s 已就绪）
test: k8s-up
    @echo "=== Running cargo nextest (auth) ==="
    cargo nextest run -p auth

# 暂停 Minikube（保留数据）
k8s-down:
    @echo "=== Stopping Minikube ==="
    minikube stop

# 查看基础设施日志
k8s-logs:
    @echo "=== Streaming all pod logs ==="
    kubectl logs -l 'app in (postgres, nats, redis, signoz-otel-collector, clickhouse, rnacos)' --tail=50 -f

# 重置 Minikube（删除并重建）
k8s-reset:
    @echo "=== Resetting Minikube ==="
    minikube delete
    minikube start --driver=docker --cpus=2 --memory=2048
    kubectl apply -k k8s/overlays/dev
    kubectl wait --for=condition=Available deployments --all --timeout=120s

# 执行 Dagger CI/CD 流水线（Go SDK 版本，需 Docker 可拉取 rust 镜像）
dagger-ci-go:
    @echo "=== Running Dagger CI pipeline (Go SDK) ==="
    dagger run go run ./ci/ ci

# 执行 Dagger CI/CD 流水线（Shell 版本，兼容无容器网络环境）
dagger-ci:
    @echo "=== Running Dagger CI pipeline (shell) ==="
    dagger run sh -c 'cd {{ justfile_directory() }} && cargo check && RUSTFLAGS="--cfg docker_tests" cargo nextest run'

# 完整 CI 流程（使用 shell 版本，兼容性更好）
ci: k8s-up
    @echo "=== Running CI pipeline (shell) ==="
    dagger run sh -c 'cd {{ justfile_directory() }} && cargo check && RUSTFLAGS="--cfg docker_tests" cargo nextest run'
    @echo "=== Cleaning up ==="
    minikube stop
