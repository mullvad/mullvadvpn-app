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
import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.repository.WireguardConstraintsRepository

class MultihopViewModel(
    private val isModal: Boolean,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
) : ViewModel() {

    val uiState: StateFlow<Lc<Boolean, MultihopUiState>> =
        wireguardConstraintsRepository.wireguardConstraints
            .filterNotNull()
            .map { Lc.Content(MultihopUiState(mode = it.multihop, isModal = isModal)) }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(isModal),
            )

    fun setMultihopMode(mode: MultihopMode) {
        viewModelScope.launch { wireguardConstraintsRepository.setMultihop(mode) }
    }
}

data class MultihopUiState(val mode: MultihopMode, val isModal: Boolean = false)
