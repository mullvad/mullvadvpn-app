package net.mullvad.mullvadvpn.feature.multihop.impl

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.repository.WireguardConstraintsRepository

class MultihopViewModel(
    private val isModal: Boolean,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
) : ViewModel() {

    val uiState: StateFlow<Lc<Boolean, MultihopUiState>> =
        wireguardConstraintsRepository.wireguardConstraints
            .filterNotNull()
            .map { Lc.Content(MultihopUiState(it.isMultihopEnabled, isModal = isModal)) }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(isModal),
            )

    fun setMultihop(enable: Boolean) {
        viewModelScope.launch { wireguardConstraintsRepository.setMultihop(enable) }
    }
}

data class MultihopUiState(val enable: Boolean, val isModal: Boolean = false)
