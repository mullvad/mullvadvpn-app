package net.mullvad.mullvadvpn.util

fun List<String>.toBulletList(): String  {
    var sb = StringBuilder()
    sb.append("<ul>")
    this.forEach {
        sb.append("<p>&nbsp;</p>\n<li><h6>&nbsp; $it</h6></li>\n")
    }
    sb.append("</ul>")
    return sb.toString()
}
