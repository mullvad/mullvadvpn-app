package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.repository.AppObfuscation
import net.mullvad.mullvadvpn.repository.AppObfuscationRepository

class AppObfuscationViewModel(private val appObfuscationRepository: AppObfuscationRepository) :
    ViewModel() {

    val uiState =
        combine(
                appObfuscationRepository.availableObfuscations,
                appObfuscationRepository.currentAppObfuscation,
            ) { availableObfuscations, currentAppObfuscation ->
                AppObfuscationUiState(
                    availableObfuscations = availableObfuscations,
                    currentAppObfuscation = currentAppObfuscation,
                )
            }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.WhileSubscribed(),
                initialValue = AppObfuscationUiState(),
            )

    fun setAppObfuscation(appObfuscation: AppObfuscation) {
        viewModelScope.launch { appObfuscationRepository.setAppObfuscation(appObfuscation) }
    }
}

data class AppObfuscationUiState(
    val availableObfuscations: List<AppObfuscation> = emptyList(),
    val currentAppObfuscation: AppObfuscation? = null,
)
