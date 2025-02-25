package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.selects.onTimeout
import kotlinx.coroutines.selects.select
import net.mullvad.mullvadvpn.constant.ACCOUNT_EXPIRY_TIMEOUT_MS
import net.mullvad.mullvadvpn.lib.common.util.isBeforeNowInstant
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.lib.shared.DeviceRepository
import net.mullvad.mullvadvpn.repository.SplashCompleteRepository
import net.mullvad.mullvadvpn.repository.UserPreferencesRepository

data class SplashScreenState(val splashComplete: Boolean = false)

class SplashViewModel(
    private val userPreferencesRepository: UserPreferencesRepository,
    private val accountRepository: AccountRepository,
    private val deviceRepository: DeviceRepository,
    private val splashCompleteRepository: SplashCompleteRepository,
) : ViewModel() {

    val uiSideEffect = flow {
        emit(getStartDestination())
        splashCompleteRepository.onSplashCompleted()
    }

    private val _uiState = MutableStateFlow(SplashScreenState(false))
    val uiState: StateFlow<SplashScreenState> = _uiState

    private suspend fun getStartDestination(): SplashUiSideEffect {
        if (!userPreferencesRepository.preferences().isPrivacyDisclosureAccepted) {
            return SplashUiSideEffect.NavigateToPrivacyDisclaimer
        }

        val deviceState =
            deviceRepository.deviceState
                .map {
                    when (it) {
                        is DeviceState.LoggedIn -> ValidStartDeviceState.LoggedIn
                        DeviceState.LoggedOut -> ValidStartDeviceState.LoggedOut
                        DeviceState.Revoked -> ValidStartDeviceState.Revoked
                        null -> null
                    }
                }
                .filterNotNull()
                .first()

        return when (deviceState) {
            ValidStartDeviceState.LoggedOut -> SplashUiSideEffect.NavigateToLogin
            ValidStartDeviceState.Revoked -> SplashUiSideEffect.NavigateToRevoked
            ValidStartDeviceState.LoggedIn -> getLoggedInStartDestination()
        }
    }

    // We know the user is logged in, but we need to find out if their account has expired
    @OptIn(ExperimentalCoroutinesApi::class)
    private suspend fun getLoggedInStartDestination(): SplashUiSideEffect {
        val expiry = viewModelScope.async { accountRepository.accountData.filterNotNull().first() }

        val accountData = select {
            expiry.onAwait { it }
            // If we don't get a response within 1 second, assume the account expiry is Missing
            onTimeout(ACCOUNT_EXPIRY_TIMEOUT_MS) { null }
        }

        return if (accountData != null && accountData.expiryDate.isBeforeNowInstant()) {
            SplashUiSideEffect.NavigateToOutOfTime
        } else {
            SplashUiSideEffect.NavigateToConnect
        }
    }
}

private sealed interface ValidStartDeviceState {
    data object LoggedIn : ValidStartDeviceState

    data object Revoked : ValidStartDeviceState

    data object LoggedOut : ValidStartDeviceState
}

sealed interface SplashUiSideEffect {
    data object NavigateToPrivacyDisclaimer : SplashUiSideEffect

    data object NavigateToRevoked : SplashUiSideEffect

    data object NavigateToLogin : SplashUiSideEffect

    data object NavigateToConnect : SplashUiSideEffect

    data object NavigateToOutOfTime : SplashUiSideEffect
}
