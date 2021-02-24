#!/bin/bash
WEBDIR=web-deploy
ASSETDIR=${WEBDIR}/assets

if [ ! -d "${WEBDIR}/" ]; then
    mkdir ${WEBDIR}
else
    rm -rf ${WEBDIR}
    mkdir ${WEBDIR}
    mkdir ${ASSETDIR}
fi

cp index.html ${WEBDIR}/index.html
cp index.js ${WEBDIR}/index.js
cp styles.css ${WEBDIR}/styles.css

cp -r pkg ${WEBDIR}/pkg
cp -r assets/deploy/* ${ASSETDIR}
