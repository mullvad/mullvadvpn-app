package net.mullvad.mullvadvpn.viewmodel

import android.os.Parcelable
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.lib.model.BuildVersion
import net.mullvad.mullvadvpn.repository.ChangelogRepository

class ChangelogViewModel(
    private val changelogRepository: ChangelogRepository,
    buildVersion: BuildVersion,
) : ViewModel() {
    private val _uiSideEffect = Channel<ChangeLogSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    val uiState: StateFlow<ChangelogUiState> =
        MutableStateFlow(
            ChangelogUiState(buildVersion.name, changelogRepository.getLastVersionChanges())
        )

    fun dismissChangelogNotification() {
        changelogRepository.setDismissNewChangelogNotification()
    }

    fun onSeeFullChangelog() =
        viewModelScope.launch { _uiSideEffect.send(ChangeLogSideEffect.OpenFullChangelog) }
}

sealed interface ChangeLogSideEffect {
    object OpenFullChangelog : ChangeLogSideEffect
}

@Parcelize data class ChangelogUiState(val version: String, val changes: List<String>) : Parcelable
