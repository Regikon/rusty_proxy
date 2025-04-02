#!/bin/sh

openssl genrsa -out ca.key 2048
openssl req -key ca.key -new -out ca.csr
openssl req -x509 -sha256 -days 365 -newkey rsa:2048 -keyout rootCA.key -out rootCA.crt
openssl x509 -req -CA rootCA.crt -CAkey rootCA.key -in ca.csr -out ca.crt -days 365 -CAcreateserial -extfile ca.ext
