package net.mullvad.mullvadvpn.feature.appinfo.impl.changelog

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ramcosta.composedestinations.generated.appinfo.destinations.ChangelogDestination
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.model.BuildVersion
import net.mullvad.mullvadvpn.lib.repository.ChangelogRepository

class ChangelogViewModel(
    private val changelogRepository: ChangelogRepository,
    savedStateHandle: SavedStateHandle,
    buildVersion: BuildVersion,
) : ViewModel() {
    private val navArgs = ChangelogDestination.argsFrom(savedStateHandle)

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
}
