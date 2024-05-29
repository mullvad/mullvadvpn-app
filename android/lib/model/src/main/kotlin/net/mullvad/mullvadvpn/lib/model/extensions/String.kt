package net.mullvad.mullvadvpn.lib.model.extensions

fun String.startCase() =
    split(" ").joinToString(" ") { word ->
        word.replaceFirstChar { firstChar -> firstChar.uppercase() }
    }
