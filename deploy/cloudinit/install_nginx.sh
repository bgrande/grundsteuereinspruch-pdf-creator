#!/bin/bash

if [ -z ${1+x} ]; then
	echo We need an SSH HOST as first parameter!
	exit 1
else
	SSH_HOST=${1}
fi

if [ -z ${2+x} ]; then
  echo We need the USER NAME as second parameter
  exit 1
else
  USER_NAME=${2}
fi

if [ -z ${3+x} ]; then
  echo We need the DOMAIN NAME as third parameter
  exit 1
else
  DOMAIN_NAME=${3}
fi

if [ -z ${4+x} ]; then
  echo We need the nginx TEMPLATE NAME as fourth parameter
  exit 1
else
  TEMPLATE_NAME=${4}
fi

if [ -z ${5+x} ]; then
  echo "We need the application's ROOT FOLDEr name as fifth parameter"
  exit 1
else
  ROOT_FOLDER=${5}
fi

if [ -z ${6+x} ]; then
  echo "We need the application's target PORT (for proxy) as sixth parameter"
  exit 1
else
  APP_PORT=${6}
fi

sed "s/_THEDOMAINNAME_/${DOMAIN_NAME}/g; s|_THEROOTFOLDER_|${ROOT_FOLDER}|g; s/_THEAPPPORT_/$APP_PORT/g" vhosts/${TEMPLATE_NAME}.conf > "./vhosts/${DOMAIN_NAME}"

/usr/bin/scp "vhosts/${DOMAIN_NAME}" ${USER_NAME}@${SSH_HOST}:~/${DOMAIN_NAME}
/usr/bin/ssh ${USER_NAME}@${SSH_HOST} "sudo cp ${DOMAIN_NAME} /etc/nginx/sites-available/${DOMAIN_NAME}"
/usr/bin/ssh ${USER_NAME}@${SSH_HOST} "if [ ! -f /etc/nginx/sites-enabled/${DOMAIN_NAME} ]; then sudo ln -s /etc/nginx/sites-available/${DOMAIN_NAME} /etc/nginx/sites-enabled/; fi"
/usr/bin/ssh ${USER_NAME}@${SSH_HOST} "if [ ! -d /var/lib/letsencrypt/.well-known ]; then sudo mkdir -p /var/lib/letsencrypt/.well-known; fi"
/usr/bin/ssh ${USER_NAME}@${SSH_HOST} "if [ ! -d /var/lib/letsencrypt/.well-known ]; then sudo chgrp www-data /var/lib/letsencrypt; fi"
/usr/bin/ssh ${USER_NAME}@${SSH_HOST} "if [ ! -d /var/lib/letsencrypt/.well-known ]; then sudo chmod g+s /var/lib/letsencrypt; fi"
/usr/bin/ssh ${USER_NAME}@${SSH_HOST} "if [ -f /etc/nginx/sites-enabled/default ]; then sudo rm /etc/nginx/sites-enabled/default; fi"
/usr/bin/ssh ${USER_NAME}@${SSH_HOST} "if [ ! -f /etc/nginx/sites-enabled/default ]; then sudo nginx -t && sudo systemctl reload nginx.service; else sudo nginx -t && sudo systemctl restart nginx.service; fi"
