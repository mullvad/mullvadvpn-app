#!/usr/bin/env bash

# This script creates or updates the existing gettext catalogues using the POT template located
# under locales/messages.pot

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
ROOT_DIR=$( dirname $SCRIPT_DIR )
POT_FILE="$ROOT_DIR/locales/messages.pot"

for PO_FILE_DIR in $ROOT_DIR/locales/* ; do
  if [ -d $PO_FILE_DIR ] ; then
    LOCALE=$( basename $PO_FILE_DIR )
    PO_FILE="$PO_FILE_DIR/messages-$LOCALE.po"

    if [ -f $PO_FILE ] ; then
      echo "Update $PO_FILE"
      msgmerge --no-fuzzy-matching --update $PO_FILE $POT_FILE
    fi
  fi
done
