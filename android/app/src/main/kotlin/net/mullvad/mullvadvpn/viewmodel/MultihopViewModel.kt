package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository

class MultihopViewModel(
    private val wireguardConstraintsRepository: WireguardConstraintsRepository
) : ViewModel() {

    val uiState: StateFlow<MultihopUiState> =
        wireguardConstraintsRepository.wireguardConstraints
            .map { MultihopUiState(it?.useMultihop ?: false) }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), MultihopUiState(false))

    fun setMultihop(enable: Boolean) {
        viewModelScope.launch { wireguardConstraintsRepository.setMultihop(enable) }
    }
}

data class MultihopUiState(val enable: Boolean)
