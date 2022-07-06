#!/bin/bash

set -ex

os=$1
triple=$2

mkdir /kerberos
curl --retry 3 -sSfL https://kerberos.org/dist/krb5/1.20/krb5-1.20.tar.gz -o krb5.tar.gz
tar -xzf /krb5.tar.gz --strip-components 1 -C /kerberos
rm /krb5.tar.gz
cd /kerberos/src

export krb5_cv_attr_constructor_destructor=yes,yes
export ac_cv_func_regcomp=yes
export ac_cv_printf_positional=yes

_configure () {
  ecode=0
  AR=${triple}-ar CC=${triple}-gcc CPP=${triple}-cpp \
    CXX=${triple}-g++ LD=${triple}-ld ./configure \
    --prefix=/usr/local/${triple} \
    --host=${triple} \
    --disable-shared \
    --enable-static \
    --without-system-verto \
    --disable-rpath \
    || ecode=$?
  cat config.log
  return $ecode
}

_configure

make -j$(nproc)
make install

