package net.mullvad.mullvadvpn.feature.multihop.impl

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.lib.usecase.MultihopInEffectStatus
import net.mullvad.mullvadvpn.lib.usecase.MultihopInEffectUseCase

class MultihopViewModel(
    private val isModal: Boolean,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    multihopInEffectUseCase: MultihopInEffectUseCase,
) : ViewModel() {

    val uiState: StateFlow<Lc<Boolean, MultihopUiState>> =
        combine(
                wireguardConstraintsRepository.wireguardConstraints.filterNotNull(),
                multihopInEffectUseCase(),
            ) { wireguardConstraints, multihopInEffect ->
                Lc.Content(
                    MultihopUiState(
                        mode = wireguardConstraints.multihop,
                        showExtraWhenNeededInfo =
                            multihopInEffect == MultihopInEffectStatus.WhenNeededInEffect,
                        isModal = isModal,
                    )
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(isModal),
            )

    fun setMultihopMode(mode: MultihopMode) {
        viewModelScope.launch { wireguardConstraintsRepository.setMultihop(mode) }
    }
}

data class MultihopUiState(
    val mode: MultihopMode,
    val showExtraWhenNeededInfo: Boolean,
    val isModal: Boolean = false,
)
