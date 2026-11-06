package net.mullvad.mullvadvpn.screen.splash

import androidx.lifecycle.ViewModel
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.common.util.isBeforeNowInstant
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.lib.repository.DeviceRepository
import net.mullvad.mullvadvpn.lib.repository.SplashCompleteRepository

class SplashViewModel(
    private val accountRepository: AccountRepository,
    private val deviceRepository: DeviceRepository,
    private val splashCompleteRepository: SplashCompleteRepository,
) : ViewModel() {

    val uiSideEffect = flow {
        emit(getStartDestination())
        splashCompleteRepository.onSplashCompleted()
    }

    private suspend fun getStartDestination(): SplashUiSideEffect {
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
        val expiryDate =
            accountRepository.fetchAccountData(timeout = ACCOUNT_EXPIRY_TIMEOUT)?.expiryDate

        return if (expiryDate != null && expiryDate.isBeforeNowInstant()) {
            SplashUiSideEffect.NavigateToOutOfTime
        } else {
            SplashUiSideEffect.NavigateToConnect
        }
    }

    companion object {
        private val ACCOUNT_EXPIRY_TIMEOUT = 1.seconds
    }
}

private sealed interface ValidStartDeviceState {
    data object LoggedIn : ValidStartDeviceState

    data object Revoked : ValidStartDeviceState

    data object LoggedOut : ValidStartDeviceState
}

sealed interface SplashUiSideEffect {
    data object NavigateToRevoked : SplashUiSideEffect

    data object NavigateToLogin : SplashUiSideEffect

    data object NavigateToConnect : SplashUiSideEffect

    data object NavigateToOutOfTime : SplashUiSideEffect
}
