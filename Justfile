# File:        Justfile
# Author:      Zakhary Kaplan <https://zakhary.dev>
# Created:     27 Apr 2022
# SPDX-License-Identifier: MIT OR Apache-2.0
# Vim:         set fdl=0 fdm=marker ft=make:

alias b := build
alias c := check
alias h := help
alias r := run
alias t := test

workspace := "--workspace --all-targets"

# default recipe
_: help

# build all artifacts
all: build doc

# compile local package
build *opts:
    @cargo build {{workspace}} {{opts}}

# check local package for errors
check:
    @cargo check {{workspace}}

# clean build artifacts
clean:
    @cargo clean

# apply lints
fix: && fmt
    @cargo clippy {{workspace}} --fix --allow-staged

# format source files
fmt:
    @cargo +nightly fmt --all

# document source files
doc:
    @cargo doc --workspace

# list available recipes
help:
    @just --list

# lint source files
lint:
    @cargo clippy {{workspace}}

# run executable
run *opts:
    @cargo run {{opts}}

# perform tests
test *opts:
    @cargo test {{workspace}} {{opts}}
