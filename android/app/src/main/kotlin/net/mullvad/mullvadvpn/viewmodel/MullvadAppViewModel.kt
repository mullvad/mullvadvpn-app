package net.mullvad.mullvadvpn.viewmodel

import android.os.Parcelable
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.launch
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.lib.model.BuildVersion
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy
import net.mullvad.mullvadvpn.repository.ChangelogRepository

class MullvadAppViewModel(
    private val changelogRepository: ChangelogRepository,
    private val connectionProxy: ConnectionProxy,
    private val buildVersion: BuildVersion,
    private val alwaysShowChangelog: Boolean,
) : ViewModel() {

    private val _uiSideEffect = MutableSharedFlow<Changelog>(replay = 1, extraBufferCapacity = 1)
    val uiSideEffect: SharedFlow<Changelog> = _uiSideEffect

    init {
        if (shouldShowChangelog()) {
            val changelog =
                Changelog(buildVersion.name, changelogRepository.getLastVersionChanges())
            viewModelScope.launch { _uiSideEffect.emit(changelog) }
        }
    }

    fun connect() {
        viewModelScope.launch { connectionProxy.connectWithoutPermissionCheck() }
    }

    private fun shouldShowChangelog(): Boolean =
        alwaysShowChangelog ||
            (changelogRepository.getVersionCodeOfMostRecentChangelogShowed() < buildVersion.code &&
                changelogRepository.getLastVersionChanges().isNotEmpty())
}

@Parcelize data class Changelog(val version: String, val changes: List<String>) : Parcelable
