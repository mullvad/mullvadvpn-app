package net.mullvad.mullvadvpn.util

fun List<String>.toBulletList(): String {
    var sb = StringBuilder()
    sb.append("<ul><p>&nbsp;</p>\n")
    this.forEach {
        sb.append("<li><h5>&nbsp; $it</h5></li>\n<p>&nbsp;</p>\n")
    }
    sb.append("</ul>")
    return sb.toString()
}
