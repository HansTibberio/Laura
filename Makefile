# Laura build for OpenBench and local use.
# OpenBench calls: make EXE=<name> CC=<compiler>  (CC is ignored for Rust)

EXE ?= laura

ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
else
	NAME := $(EXE)
endif

openbench:
	cargo rustc --release --bin laura -- -C target-cpu=native --emit link=$(NAME)

.PHONY: openbench
