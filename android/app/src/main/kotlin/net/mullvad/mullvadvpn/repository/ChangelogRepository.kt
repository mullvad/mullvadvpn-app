package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.model.BuildVersion
import net.mullvad.mullvadvpn.lib.repository.UserPreferencesRepository
import net.mullvad.mullvadvpn.util.IChangelogDataProvider
import net.mullvad.mullvadvpn.util.trimAll

private const val NEWLINE_CHAR = '\n'
private const val BULLET_POINT_CHAR = '-'

class ChangelogRepository(
    private val dataProvider: IChangelogDataProvider,
    private val userPreferencesRepository: UserPreferencesRepository,
    private val buildVersion: BuildVersion,
    dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    val hasUnreadChangelog: StateFlow<Boolean> =
        userPreferencesRepository
            .preferencesFlow()
            .map {
                getLastVersionChanges().isNotEmpty() &&
                    buildVersion.code > it.lastShownChangelogVersionCode
            }
            .stateIn(
                CoroutineScope(dispatcher),
                started = SharingStarted.Eagerly,
                initialValue = false,
            )

    suspend fun setDismissNewChangelogNotification() {
        userPreferencesRepository.setHasDisplayedChangelogNotification()
    }

    fun getLastVersionChanges(): List<String> =
        // Prepend with a new line char so each entry consists of NEWLINE_CHAR + BULLET_POINT_CHAR
        (NEWLINE_CHAR + dataProvider.getChangelog())
            .split(NEWLINE_CHAR.toString() + BULLET_POINT_CHAR)
            .trimAll()
            .filter { it.isNotEmpty() }
}
