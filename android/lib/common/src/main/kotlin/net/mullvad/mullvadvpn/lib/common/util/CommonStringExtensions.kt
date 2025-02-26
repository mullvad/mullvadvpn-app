package net.mullvad.mullvadvpn.lib.common.util

private const val BIG_DOT_CHAR = "●"
private const val SPACE_CHAR = ' '

fun String.groupWithSpaces(groupCharSize: Int = 4): String {
    return fold(StringBuilder()) { formattedText, nextDigit ->
            if ((formattedText.length % (groupCharSize + 1)) == groupCharSize) {
                formattedText.append(SPACE_CHAR)
            }
            formattedText.append(nextDigit)
        }
        .toString()
}

fun String.groupPasswordModeWithSpaces(groupCharSize: Int = 4): String {
    return BIG_DOT_CHAR.repeat(this.length).groupWithSpaces(groupCharSize)
}
