package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.repository.AppObfuscation
import net.mullvad.mullvadvpn.repository.AppObfuscationRepository
import net.mullvad.mullvadvpn.util.Lc

class AppearanceViewModel(private val appObfuscationRepository: AppObfuscationRepository) :
    ViewModel() {

    val uiState =
        combine(
                appObfuscationRepository.availableObfuscations,
                appObfuscationRepository.currentAppObfuscation,
            ) { availableObfuscations, currentAppObfuscation ->
                Lc.Content(
                    AppearanceUiState(
                        availableObfuscations = availableObfuscations,
                        currentAppObfuscation = currentAppObfuscation,
                    )
                )
            }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.WhileSubscribed(),
                initialValue = Lc.Loading(Unit),
            )

    fun setAppObfuscation(appObfuscation: AppObfuscation) {
        viewModelScope.launch { appObfuscationRepository.setAppObfuscation(appObfuscation) }
    }
}

data class AppearanceUiState(
    val availableObfuscations: List<AppObfuscation> = emptyList(),
    val currentAppObfuscation: AppObfuscation? = null,
)
