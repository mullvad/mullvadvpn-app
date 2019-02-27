#!/usr/bin/env bash

# This script creates or updates the existing gettext catalogues using the POT template located
# under locales/messages.pot

ROOT_DIR=$(dirname $(dirname "${BASH_SOURCE[0]}"))
POT_FILE="$ROOT_DIR/locales/messages.pot"

for PO_FILE_DIR in $ROOT_DIR/locales/* ; do
  if [ -d $PO_FILE_DIR ] ; then
    PO_FILE="$PO_FILE_DIR/messages.po"
    GITKEEP_FILE="$PO_FILE_DIR/.gitkeep"

    if [ -f $PO_FILE ] ; then
      echo "Update $PO_FILE_DIR\c"
      msgmerge --no-fuzzy-matching --update $PO_FILE $POT_FILE
    else
      if [ -f $GITKEEP_FILE ] ; then
        echo "Remove $GITKEEP_FILE to initialize the new translation"
      else
        msginit --input $POT_FILE --output $PO_FILE --no-translator
      fi
    fi
  fi
done
