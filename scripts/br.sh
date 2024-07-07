#!/bin/sh

cargo build
./run configs/simple-1.json --replica target/debug/raft_rs
