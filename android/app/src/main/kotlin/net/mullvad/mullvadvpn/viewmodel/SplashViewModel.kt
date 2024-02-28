package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.selects.onTimeout
import kotlinx.coroutines.selects.select
import net.mullvad.mullvadvpn.constant.ACCOUNT_EXPIRY_TIMEOUT_MS
import net.mullvad.mullvadvpn.model.AccountAndDevice
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.repository.PrivacyDisclaimerRepository

class SplashViewModel(
    private val privacyDisclaimerRepository: PrivacyDisclaimerRepository,
    private val deviceRepository: DeviceRepository,
    private val accountRepository: AccountRepository,
) : ViewModel() {

    val uiSideEffect = flow { emit(getStartDestination()) }

    private suspend fun getStartDestination(): SplashUiSideEffect {
        if (!privacyDisclaimerRepository.hasAcceptedPrivacyDisclosure()) {
            return SplashUiSideEffect.NavigateToPrivacyDisclaimer
        }

        val deviceState =
            deviceRepository.deviceState
                .map {
                    when (it) {
                        DeviceState.Initial -> null
                        is DeviceState.LoggedIn ->
                            ValidStartDeviceState.LoggedIn(it.accountAndDevice)
                        DeviceState.LoggedOut -> ValidStartDeviceState.LoggedOut
                        DeviceState.Revoked -> ValidStartDeviceState.Revoked
                        DeviceState.Unknown -> null
                    }
                }
                .filterNotNull()
                .first()

        return when (deviceState) {
            ValidStartDeviceState.LoggedOut -> SplashUiSideEffect.NavigateToLogin
            ValidStartDeviceState.Revoked -> SplashUiSideEffect.NavigateToRevoked
            is ValidStartDeviceState.LoggedIn -> getLoggedInStartDestination()
        }
    }

    // We know the user is logged in, but we need to find out if their account has expired
    private suspend fun getLoggedInStartDestination(): SplashUiSideEffect {
        val expiry =
            viewModelScope.async {
                accountRepository.accountExpiryState.first { it !is AccountExpiry.Missing }
            }

        val accountExpiry = select {
            expiry.onAwait { it }
            // If we don't get a response within 1 second, assume the account expiry is Missing
            onTimeout(ACCOUNT_EXPIRY_TIMEOUT_MS) { AccountExpiry.Missing }
        }

        return when (accountExpiry) {
            is AccountExpiry.Available -> {
                if (accountExpiry.expiryDateTime.isBeforeNow) {
                    SplashUiSideEffect.NavigateToOutOfTime
                } else {
                    SplashUiSideEffect.NavigateToConnect
                }
            }
            AccountExpiry.Missing -> SplashUiSideEffect.NavigateToConnect
        }
    }
}

private sealed interface ValidStartDeviceState {
    data class LoggedIn(val accountAndDevice: AccountAndDevice) : ValidStartDeviceState

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
