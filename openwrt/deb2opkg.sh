#!/bin/bash
# inspired from https://github.com/jordansissel/fpm/issues/1323#issuecomment-450613806

if [ $# -ne 2 ]; then
    echo 'USAGE: ./dep2opkg.sh arch input-deb'
    echo 'WARNING: for amd64 set arch as x86_64'
    exit 1
fi

set -e
set -x

TMP_PATH=`mktemp -d`
cp $2 $TMP_PATH
pushd $TMP_PATH

DEB_NAME=`ls *.deb`
ar x $DEB_NAME
rm -rf $DEB_NAME

mkdir control
pushd control
tar xf ../control.tar.gz
sed "s/Architecture:\\ \w*/Architecture:\\ $1/g" ./control -i
cat control
tar czf ../control.tar.gz ./*

popd
tar czf $DEB_NAME.ipk control.tar.gz data.tar.gz debian-binary
#rename s/.deb.ipk/_"$1".ipk/ "$DEB_NAME".ipk
prename s/.deb.ipk/_"$1".ipk/ "$DEB_NAME".ipk

popd
cp $TMP_PATH/*.ipk .

# opkg install mything.ipk
