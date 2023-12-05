package net.mullvad.mullvadvpn.viewmodel

import android.os.Parcelable
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.launch
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.repository.ChangelogRepository

class ChangelogViewModel(
    private val changelogRepository: ChangelogRepository,
    private val buildVersionCode: Int,
    private val alwaysShowChangelog: Boolean
) : ViewModel() {

    private val _uiSideEffect = MutableSharedFlow<Changelog>(replay = 1, extraBufferCapacity = 1)
    val uiSideEffect: SharedFlow<Changelog> = _uiSideEffect

    init {
        if (shouldShowChangelog()) {
            val changelog =
                Changelog(BuildConfig.VERSION_NAME, changelogRepository.getLastVersionChanges())
            viewModelScope.launch { _uiSideEffect.emit(changelog) }
        }
    }

    fun markChangelogAsRead() {
        changelogRepository.setVersionCodeOfMostRecentChangelogShowed(buildVersionCode)
    }

    private fun shouldShowChangelog(): Boolean =
        alwaysShowChangelog ||
            (changelogRepository.getVersionCodeOfMostRecentChangelogShowed() < buildVersionCode &&
                changelogRepository.getLastVersionChanges().isNotEmpty())
}

@Parcelize data class Changelog(val version: String, val changes: List<String>) : Parcelable
