package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ramcosta.composedestinations.generated.destinations.DaitaDestination
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.DaitaUiState
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.repository.SettingsRepository

class DaitaViewModel(
    private val settingsRepository: SettingsRepository,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val navArgs = DaitaDestination.argsFrom(savedStateHandle)

    val uiState =
        settingsRepository.settingsUpdates
            .map { settings ->
                DaitaUiState(
                    daitaEnabled = settings?.daitaSettings()?.enabled == true,
                    directOnly = settings?.daitaSettings()?.directOnly == true,
                    navArgs.isModal,
                )
            }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.WhileSubscribed(),
                initialValue = DaitaUiState(daitaEnabled = false, directOnly = false),
            )

    fun setDaita(enable: Boolean) {
        viewModelScope.launch { settingsRepository.setDaitaEnabled(enable) }
    }

    fun setDirectOnly(enable: Boolean) {
        viewModelScope.launch { settingsRepository.setDaitaDirectOnly(enable) }
    }

    private fun Settings.daitaSettings() = tunnelOptions.wireguard.daitaSettings
}
