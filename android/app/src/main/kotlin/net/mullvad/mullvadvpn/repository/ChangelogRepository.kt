package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import net.mullvad.mullvadvpn.util.IChangelogDataProvider
import net.mullvad.mullvadvpn.util.trimAll

private const val NEWLINE_CHAR = '\n'
private const val BULLET_POINT_CHAR = '-'

class ChangelogRepository(private val dataProvider: IChangelogDataProvider) {
    private val _showNewChangelogNotification = MutableStateFlow(false)

    val showNewChangelogNotification: StateFlow<Boolean> = _showNewChangelogNotification

    fun setShowNewChangelogNotification() {
        _showNewChangelogNotification.value = true
    }

    fun setDismissNewChangelogNotification() {
        _showNewChangelogNotification.value = false
    }

    fun getLastVersionChanges(): List<String> =
        // Prepend with a new line char so each entry consists of NEWLINE_CHAR + BULLET_POINT_CHAR
        (NEWLINE_CHAR + dataProvider.getChangelog())
            .split(NEWLINE_CHAR.toString() + BULLET_POINT_CHAR)
            .trimAll()
            .filter { it.isNotEmpty() }
}
