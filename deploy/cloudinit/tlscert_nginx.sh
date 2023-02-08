#!/bin/bash

if [ -z ${1+x} ]; then
	echo We need an ssh host as first parameter!
	exit 1
else
	SSH_HOST=${1}
fi

if [ -z ${2+x} ]; then
  echo We need the user name as second parameter
  exit 1
else
  USER_NAME=${2}
fi

if [ -z ${3+x} ]; then
  echo We need the ssl host name as third parameter
  exit 1
else
  SSL_HOST=${3}
fi

if [ -z ${4+x} ]; then
  IS_WWW=0
else
  IS_WWW=1
fi

DOMAIN_LIST="-d \"${SSL_HOST}\""
if [ ${IS_WWW} = 1 ]; then
  DOMAIN_LIST="$DOMAIN_LIST -d \"www.${SSL_HOST}\""
fi

/usr/bin/ssh ${USER_NAME}@${SSH_HOST} "if [ ! -f /etc/ssl/certs/dhparam.pem ]; then sudo openssl dhparam -out /etc/ssl/certs/dhparam.pem 2048; fi"
/usr/bin/ssh ${USER_NAME}@${SSH_HOST} "sudo certbot certonly --agree-tos --email sslle@bgrande.de --webroot -w /var/lib/letsencrypt/ ${DOMAIN_LIST}"
/usr/bin/ssh ${USER_NAME}@${SSH_HOST} "sudo systemctl restart nginx.service"