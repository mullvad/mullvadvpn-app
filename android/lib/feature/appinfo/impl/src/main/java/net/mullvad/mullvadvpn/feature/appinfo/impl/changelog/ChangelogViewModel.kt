package net.mullvad.mullvadvpn.feature.appinfo.impl.changelog

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.feature.appinfo.api.ChangelogNavKey
import net.mullvad.mullvadvpn.lib.model.BuildVersion
import net.mullvad.mullvadvpn.lib.repository.ChangelogRepository

class ChangelogViewModel(
    navArgs: ChangelogNavKey,
    private val changelogRepository: ChangelogRepository,
    buildVersion: BuildVersion,
) : ViewModel() {

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
