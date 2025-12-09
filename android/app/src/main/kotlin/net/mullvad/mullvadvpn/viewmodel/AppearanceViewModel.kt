package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.repository.AppObfuscation
import net.mullvad.mullvadvpn.repository.AppObfuscationRepository
import net.mullvad.mullvadvpn.util.Lc

class AppearanceViewModel(private val appObfuscationRepository: AppObfuscationRepository) :
    ViewModel() {

    private val applying = MutableStateFlow(false)

    val uiState =
        combine(
                appObfuscationRepository.availableObfuscations,
                appObfuscationRepository.currentAppObfuscation,
                applying,
            ) { availableObfuscations, currentAppObfuscation, applying ->
                Lc.Content(
                    AppearanceUiState(
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
            applying.emit(true)
            appObfuscationRepository.setAppObfuscation(appObfuscation)
        }
    }
}

data class AppearanceUiState(
    val availableObfuscations: List<AppObfuscation> = emptyList(),
    val currentAppObfuscation: AppObfuscation? = null,
    val applyingChange: Boolean = false,
)
