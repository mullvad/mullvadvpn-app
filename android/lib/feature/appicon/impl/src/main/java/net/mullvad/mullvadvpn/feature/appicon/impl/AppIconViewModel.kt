package net.mullvad.mullvadvpn.feature.appicon.impl

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.feature.appicon.impl.obfuscation.AppObfuscation
import net.mullvad.mullvadvpn.feature.appicon.impl.obfuscation.AppObfuscationRepository
import net.mullvad.mullvadvpn.lib.common.Lc

class AppIconViewModel(private val appObfuscationRepository: AppObfuscationRepository) :
    ViewModel() {

    private val applying = MutableStateFlow(false)

    val uiState =
        combine(
                appObfuscationRepository.availableObfuscations,
                appObfuscationRepository.currentAppObfuscation,
                applying,
            ) { availableObfuscations, currentAppObfuscation, applying ->
                Lc.Content(
                    AppIconUiState(
                        availableObfuscations = availableObfuscations,
                        currentAppObfuscation = currentAppObfuscation,
                        applyingChange = applying,
                    )
                )
            }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.WhileSubscribed(),
                initialValue = Lc.Loading(Unit),
            )

    fun setAppObfuscation(appObfuscation: AppObfuscation) {
        viewModelScope.launch {
            if (appObfuscation != appObfuscationRepository.currentAppObfuscation.value) {
                applying.emit(true)
                appObfuscationRepository.setAppObfuscation(appObfuscation)
            }
        }
    }
}
