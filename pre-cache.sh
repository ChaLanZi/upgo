#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────
# pre-cache.sh — Pre-cache infrastructure images into Minikube
#
# Usage:
#   ./pre-cache.sh              # Pull + load all images
#   ./pre-cache.sh list         # List cached images
#   ./pre-cache.sh pull         # Pull only (host)
#   ./pre-cache.sh load         # Load only (host→minikube)
#
# Why: In China, Docker Hub is unreachable from Minikube's
# internal Docker daemon. Pre-caching avoids ImagePullBackOff.
# ──────────────────────────────────────────────────────────
set -euo pipefail

IMAGES=(
  "postgres:16"
  "redis:7"
  "nats:2.10"
  "axllent/mailpit:latest"
  "qingpan/rnacos:latest"
  "clickhouse/clickhouse-server:24.3"
  "signoz/signoz-otel-collector:0.88.17"
)

pull_images() {
  echo "=== Pulling images (host) ==="
  for img in "${IMAGES[@]}"; do
    echo "  Pulling $img..."
    docker pull "$img" || echo "  ⚠️  Failed to pull $img"
  done
}

load_images() {
  echo "=== Loading images into Minikube ==="
  for img in "${IMAGES[@]}"; do
    name=$(echo "$img" | tr '/' '_' | tr ':' '_')
    echo "  Loading $img..."
    docker save "$img" -o "/tmp/infra_${name}.tar" 2>/dev/null
    docker cp "/tmp/infra_${name}.tar" minikube:"/infra_${name}.tar" 2>/dev/null
    docker exec minikube docker load -i "/infra_${name}.tar" 2>/dev/null || \
      echo "  ⚠️  Failed to load $img (Minikube running?)"
  done
  echo "=== Verify ==="
  minikube image ls 2>/dev/null || echo "  (Minikube not running)"
}

list_images() {
  echo "=== Cached images in Minikube ==="
  minikube image ls 2>/dev/null || echo "  (Minikube not running)"
  echo ""
  echo "=== Images on host ==="
  for img in "${IMAGES[@]}"; do
    echo "  $img: $(docker images --format '{{.Size}}' "$img" 2>/dev/null || echo 'not cached')"
  done
}

case "${1:-all}" in
  pull)   pull_images ;;
  load)   load_images ;;
  list)   list_images ;;
  all|*)
    pull_images
    load_images
    list_images
    ;;
esac
