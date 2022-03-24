package net.mullvad.mullvadvpn.util

fun String.capitalizeFirstCharOfEachWord(): String {
    return split(" ")
        .joinToString(" ") { word ->
            word.replaceFirstChar { firstChar -> firstChar.uppercase() }
        }
        .trimEnd()
}
