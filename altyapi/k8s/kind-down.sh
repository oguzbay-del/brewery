#!/usr/bin/env bash
# kind kümesini siler. Kullanım: kökte `make kind-down`.
set -euo pipefail
kind delete cluster --name demleme
