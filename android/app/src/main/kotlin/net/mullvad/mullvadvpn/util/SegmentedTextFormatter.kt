package net.mullvad.mullvadvpn.util

class SegmentedTextFormatter(var separator: Char) {
    var isValidInputCharacter: (Char) -> Boolean = { _ -> true }
    var segmentSize = 4

    fun format(string: String) = string
        .asSequence()
        .filter(isValidInputCharacter)
        .chunked(segmentSize)
        .map { segmentCharacters -> segmentCharacters.joinToString("") }
        .joinToString("$separator")
}
