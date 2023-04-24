#!/usr/bin/env bash

PREFIX=$(date +%Y%m%d)
BACKUP_FILE=backup_before_${PREFIX}-gsonline.tar.bz2

function doBackup() {
	B_PATH=$1
	BACKUP_FILE=$2
	BACKUP_DIR=$3
	FILES_TO_BACKUP=$4

	echo backuping...

	ssh -i ${B_PATH}/deploy/gseonline_deploy_id_rsa \
	    -o IdentitiesOnly=yes \
	    pdfcreator@app.grundsteuereinspruch.online \
	    "mkdir -p ${BACKUP_DIR} && tar cjf  --exclude='*.mov' --exclude='*.ogv' ${BACKUP_DIR}/${BACKUP_FILE} ${FILES_TO_BACKUP}"
	echo ...backup done
}

function getBackup() {
  SRC_DIR=$1
  TARGET_DIR=$2
  BACKUP_FILE=$3

  mkdir -p ${TARGET_DIR}

  scp -i deploy/gseonline_deploy_id_rsa \
      -o IdentitiesOnly=yes \
      pdfcreator@app.grundsteuereinspruch.online:${SRC_DIR}/${BACKUP_FILE} \
      ${TARGET_DIR}/${BACKUP_FILE}
}

doBackup "${PWD}" "${BACKUP_FILE}" "~/backup" app && getBackup "~/backup" "${PWD}/backup" ${BACKUP_FILE}