package net.mullvad.mullvadvpn.feature.lansharing.impl

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository

class LocalNetworkSharingViewModel(
    private val isModal: Boolean,
    private val settingsRepository: SettingsRepository,
) : ViewModel() {

    val uiState =
        settingsRepository.settingsUpdates
            .filterNotNull()
            .map { settings ->
                LocalNetworkSharingUiState(lanSharingEnabled = settings.allowLan, isModal)
                    .toLc<Boolean, LocalNetworkSharingUiState>()
            }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                initialValue = Lc.Loading(isModal),
            )

    fun setLocalNetworkSharingEnabled(enable: Boolean) {
        viewModelScope.launch { settingsRepository.setLocalNetworkSharing(enable) }
    }
}
