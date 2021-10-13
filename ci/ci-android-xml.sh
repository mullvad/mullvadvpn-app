# CI/Developer script to format
# Relies on Tidy - https://github.com/htacg/tidy-html5


# Autoformats Android XML files
function tidy-up-android-xml {
    tidy -xml \
        -m  \
        -i  \
        -w 100 \
        -utf8 \
        --quiet yes \
        --indent-attributes yes \
        --indent-spaces 4 \
        --literal-attributes yes \
        android/app/src/main/AndroidManifest.xml \
        android/app/src/main/res/anim*/*.xml \
        android/app/src/main/res/drawable*/*.xml \
        android/app/src/main/res/layout*/*.xml \
        android/app/src/main/res/values/*.xml

    # FIXME - when tidy learns to not leave whitespace around, remove the line below - https://github.com/htacg/tidy-html5/issues/864
    find android/app/src/main/ -name '*.xml' -exec sed -i -e 's/[ \t]*$//' '{}' ';'
}

# Autoformats Android XML files and returns 0 if no files were actually changed, or 1 if files were changed
function tidy-verify-xml {
    tidy-up-android-xml

    if git diff --exit-code -- android/app/src/main/AndroidManifest.xml android/app/src/main/res; then
        echo "Android XML files are correctly formatted"
        return 0
    else
        echo "android/app/src/main contains files that were changed, XML is not formatted properly"
        return 1
    fi
}
