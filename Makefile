SHELL := /usr/bin/env bash

.DEFAULT_GOAL := build
EXAMPLES := pumpfun_trade pumpfun_trade_with_safekey pumpswap_trade pumpswap_trade_with_safekey

.PHONY: fmt check build clean

fmt:
	@set -e; for d in $(EXAMPLES); do echo "==> cargo fmt $$d"; (cd "$$d" && cargo fmt); done

check:
	@set -e; for d in $(EXAMPLES); do echo "==> cargo check $$d"; (cd "$$d" && cargo check); done

build:
	@set -e; for d in $(EXAMPLES); do echo "==> cargo build --release $$d"; (cd "$$d" && cargo build --release); done

clean:
	@set -e; for d in $(EXAMPLES); do echo "==> cargo clean $$d"; (cd "$$d" && cargo clean); done
