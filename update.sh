#!/bin/bash

if [ -z "$1" ]
then
    echo "no command given"
    exit 1
elif [ "$1" != "release" ] && [ "$1" != "debug" ]
then
    echo "unknown update command given: $1"
    exit 1
fi

git pull
npm install

if [ "$1" = "release" ]
then
    cargo build --bins --release
    npm run webpack-production
elif [ "$1" == "debug" ]
then
    cargo build --bins
    npm run webpack
fi