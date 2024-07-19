package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.compose.state.ImportOverridesSheetUiState
import net.mullvad.mullvadvpn.repository.RelayOverridesRepository

class ImportOverridesSheetViewModel(
    serverIpOverridesRepository: RelayOverridesRepository,
) : ViewModel() {

    val uiState =
        serverIpOverridesRepository.relayOverrides
            .map { it?.isNotEmpty() == true }
            .map { ImportOverridesSheetUiState(overridesActive = it) }
            .stateIn(
                viewModelScope,
                started = SharingStarted.WhileSubscribed(),
                ImportOverridesSheetUiState(overridesActive = false)
            )
}
