#!/bin/bash
WEBDIR=web-deploy

if [ ! -d "${WEBDIR}/" ]; then
    mkdir ${WEBDIR}
else
    rm -rf ${WEBDIR}
    mkdir ${WEBDIR}
fi

cp index.html ${WEBDIR}/index.html

cp -r pkg ${WEBDIR}/pkg
