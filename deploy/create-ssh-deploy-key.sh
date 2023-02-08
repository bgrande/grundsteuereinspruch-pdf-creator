#!/bin/bash

if [ -z ${1+x} ]; then
	echo We need a target name as first parameter!
	exit 1
else
	NAME=${1}
fi

ssh-keygen -P "" -t rsa -b 4096 -C "${NAME}deploy@bgrande.de" -f "${NAME}_deploy_id_rsa"
