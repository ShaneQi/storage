#!/bin/bash
docker run \
-d \
--name storage \
-v `pwd`:/storage_server \
-v /home/shane/persistence/public/storage:/storage \
-w /storage_server \
rust:1.20 \
/bin/sh -c \
"\
cargo run /storage;
"
