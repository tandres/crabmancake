#!/bin/bash
TOKEN=${ACCESS_TOKEN}
REV=$(git rev-parse --short HEAD)
WEBDIR=web-deploy
WORKDIR=/tmp/crabmancake_deploy_${REV}
TARGET_REPO=https://${TOKEN}@github.com/tandres/tandres.github.io

echo git clone -b master ${TARGET_REPO} ${WORKDIR}
git clone -b master ${TARGET_REPO} ${WORKDIR}

cp -r ${WEBDIR}/* ${WORKDIR}

pushd .
cd ${WORKDIR}
git config user.name "crabmancake_deploy"
git config user.email "crabmancake@gmail.com"
git add .
git commit --allow-empty -m "Automated deploy for crabmancake ${REV}"
git push ${TARGET_REPO} master
popd
