package net.mullvad.mullvadvpn.viewmodel

import android.os.Parcelable
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ramcosta.composedestinations.generated.destinations.ChangelogDestination
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
    savedStateHandle: SavedStateHandle,
    buildVersion: BuildVersion,
) : ViewModel() {
    private val navArgs = ChangelogDestination.argsFrom(savedStateHandle)
    private val _uiSideEffect = Channel<ChangeLogSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    val uiState: StateFlow<ChangelogUiState> =
        MutableStateFlow(
            ChangelogUiState(
                navArgs.isModal,
                buildVersion.name,
                changelogRepository.getLastVersionChanges(),
            )
        )

    fun dismissChangelogNotification() =
        viewModelScope.launch { changelogRepository.setDismissNewChangelogNotification() }

    fun onSeeFullChangelog() =
        viewModelScope.launch { _uiSideEffect.send(ChangeLogSideEffect.OpenFullChangelog) }
}

sealed interface ChangeLogSideEffect {
    object OpenFullChangelog : ChangeLogSideEffect
}

@Parcelize
data class ChangelogUiState(
    val isModal: Boolean = false,
    val version: String,
    val changes: List<String>,
) : Parcelable
