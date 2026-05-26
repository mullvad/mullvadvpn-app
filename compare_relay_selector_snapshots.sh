#!/bin/bash

for source in mullvad-relay-selector/tests/snapshots/*.snap; do
    filename=$(basename "$source")
    compare="desktop/packages/mullvad-vpn/$filename"
    cp "$source" tmpfile
    sed -i '1,4d' tmpfile
    sed -i -e "s/Discarded(IncludeInCountry)/Match/g" tmpfile
    sed -i -e "s/\ IncludeInCountry//g" tmpfile
    sed -i -e "s/IncludeInCountry\ //g" tmpfile
    diff tmpfile $compare > /dev/null
    if [ "$?" -ne 0 ]; then
        echo "SNAPSHOTS ARE NOT THE SAME: $filename"
        echo "File 1 $source"
        echo "File 2 $compare"
        diff tmpfile $compare
        break
    else
        echo "SNAPSHOTS MATCH: $filename"
    fi
done

rm tmpfile
