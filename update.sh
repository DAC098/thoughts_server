#!/bin/bash

git pull
cargo build --bins --release
npm run webpack