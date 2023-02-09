#!/bin/bash
cp ${PWD}/.env.prod build/.env
mkdir -p build/data build/logs
cp -r data/db data/templates build/data/
cargo test
# link openssl statically -> from https://users.rust-lang.org/t/how-to-link-openssl-statically/14912/2
OPENSSL_STATIC=yes OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu OPENSSL_INCLUDE_DIR=/usr/include/ cargo build -r && cp target/release/pdf-creator-shuttle build/pdfcreator
# from https://hub.docker.com/_/rust
#docker run --rm --user "$(id -u)":"$(id -g)" -v "$PWD":/usr/src/pdfcreator -w /usr/src/pdfcreator rust:1.67.0 cargo build --release && cp target/release/pdf-creator-shuttle build/pdfcreator

rsync \
  -avzr \
  --delete \
  -e "ssh -i deploy/gseonline_deploy_id_rsa -o IdentitiesOnly=yes" \
  build/ \
  pdfcreator@app.grundsteuereinspruch.online:/home/pdfcreator/app

ssh -i deploy/gseonline_deploy_id_rsa -o IdentitiesOnly=yes pdfcreator@app.grundsteuereinspruch.online 'sudo /bin/systemctl restart pdfcreator'