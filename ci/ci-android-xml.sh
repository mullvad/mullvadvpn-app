# CI/Developer script to format
# Relies on Tidy - https://github.com/htacg/tidy-html5


# Autoformats Android XML files
function tidy-up-android-xml {

    tidy -xml \
        -m  \
        -i  \
        --quiet yes \
        --indent-attributes yes \
        --indent-spaces 4 \
        --literal-attributes yes \
        android/src/main/AndroidManifest.xml android/src/main/res/*/*.xml
    # FIXME - when tidy learns to not leave whitespace around, remove the line below - https://github.com/htacg/tidy-html5/issues/864
    find android/src/main/ -name '*.xml' -exec sed -i -e 's/[ \t]*$//' '{}' ';'
}

# Autoformats Android XML files and returns 0 if no files were actually changed, or 1 if files were changed
function tidy-verify-xml {
    tidy-up-android-xml
    if (( $(git diff android/src/main/AndroidManifest.xml android/src/main/res/ | wc -l) > 0 )); then
        echo "android/src/main contains files that were changed, XML is not formatted properly"
        git diff -- android/src/main/AndroidManifest.xml android/src/main/res
        return 1;
    else
        echo "Android XML files are correctly formatted"
        return 0;
    fi
}
