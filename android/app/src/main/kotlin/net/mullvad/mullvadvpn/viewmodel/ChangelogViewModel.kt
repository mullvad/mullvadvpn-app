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

    private val _uiSideEffect = MutableSharedFlow<ChangeLog>(replay = 1, extraBufferCapacity = 1)
    val uiSideEffect: SharedFlow<ChangeLog> = _uiSideEffect

    init {
        if (shouldShowChangeLog()) {
            val changeLog =
                ChangeLog(BuildConfig.VERSION_NAME, changelogRepository.getLastVersionChanges())
            viewModelScope.launch { _uiSideEffect.emit(changeLog) }
        }
    }

    fun markChangeLogAsRead() {
        changelogRepository.setVersionCodeOfMostRecentChangelogShowed(buildVersionCode)
    }

    private fun shouldShowChangeLog(): Boolean =
        alwaysShowChangelog ||
            (changelogRepository.getVersionCodeOfMostRecentChangelogShowed() < buildVersionCode &&
                changelogRepository.getLastVersionChanges().isNotEmpty())
}

@Parcelize data class ChangeLog(val version: String, val changes: List<String>) : Parcelable
