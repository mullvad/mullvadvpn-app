package net.mullvad.mullvadvpn.repository

import net.mullvad.mullvadvpn.util.IChangelogDataProvider
import net.mullvad.mullvadvpn.util.trimAll

private const val NEWLINE_CHAR = '\n'
private const val BULLET_POINT_CHAR = '-'

class ChangelogRepository(private val dataProvider: IChangelogDataProvider) {

    fun getLastVersionChanges(): List<String> =
        // Prepend with a new line char so each entry consists of NEWLINE_CHAR + BULLET_POINT_CHAR
        (NEWLINE_CHAR + dataProvider.getChangelog())
            .split(NEWLINE_CHAR.toString() + BULLET_POINT_CHAR)
            .trimAll()
            .filter { it.isNotEmpty() }
}
