package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ramcosta.composedestinations.generated.destinations.DaitaDestination
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.DaitaUiState
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.isDaitaDirectOnly
import net.mullvad.mullvadvpn.util.isDaitaEnabled
import net.mullvad.mullvadvpn.util.toLc

class DaitaViewModel(
    private val settingsRepository: SettingsRepository,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val navArgs = DaitaDestination.argsFrom(savedStateHandle)

    val uiState =
        settingsRepository.settingsUpdates
            .filterNotNull()
            .map { settings ->
                DaitaUiState(
                        daitaEnabled = settings.isDaitaEnabled(),
                        directOnly = settings.isDaitaDirectOnly(),
                        navArgs.isModal,
                    )
                    .toLc<Boolean, DaitaUiState>()
            }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                initialValue = Lc.Loading(navArgs.isModal),
            )

    fun setDaita(enable: Boolean) {
        viewModelScope.launch { settingsRepository.setDaitaEnabled(enable) }
    }

    fun setDirectOnly(enable: Boolean) {
        viewModelScope.launch { settingsRepository.setDaitaDirectOnly(enable) }
    }
}
