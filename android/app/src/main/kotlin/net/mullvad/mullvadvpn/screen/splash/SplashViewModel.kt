package net.mullvad.mullvadvpn.screen.splash

import androidx.lifecycle.ViewModel
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.mapNotNull
import net.mullvad.mullvadvpn.feature.onboarding.api.GoodToKnowStep
import net.mullvad.mullvadvpn.feature.onboarding.api.OnboardingStep
import net.mullvad.mullvadvpn.lib.common.util.isBeforeNowInstant
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.lib.repository.DeviceRepository
import net.mullvad.mullvadvpn.lib.repository.SplashCompleteRepository
import net.mullvad.mullvadvpn.lib.userpreferences.OnboardingCheckpoint
import net.mullvad.mullvadvpn.lib.userpreferences.UserPreferencesRepository

class SplashViewModel(
    private val accountRepository: AccountRepository,
    private val deviceRepository: DeviceRepository,
    private val splashCompleteRepository: SplashCompleteRepository,
    private val preferencesRepository: UserPreferencesRepository,
) : ViewModel() {

    val uiSideEffect = flow {
        emit(getStartDestination())
        splashCompleteRepository.onSplashCompleted()
    }

    private suspend fun getStartDestination(): SplashUiSideEffect {
        val deviceState =
            deviceRepository.deviceState
                .mapNotNull {
                    when (it) {
                        is DeviceState.LoggedIn -> ValidStartDeviceState.LoggedIn
                        DeviceState.LoggedOut -> ValidStartDeviceState.LoggedOut
                        DeviceState.Revoked -> ValidStartDeviceState.Revoked
                        null -> null
                    }
                }
                .first()

        return when (deviceState) {
            ValidStartDeviceState.LoggedOut -> SplashUiSideEffect.NavigateToLogin
            ValidStartDeviceState.Revoked -> SplashUiSideEffect.NavigateToRevoked
            ValidStartDeviceState.LoggedIn -> getOnboardingOrLoggedInDestination()
        }
    }

    private suspend fun getOnboardingOrLoggedInDestination(): SplashUiSideEffect =
        when (preferencesRepository.onboardingCheckpoint().first()) {
            OnboardingCheckpoint.NOT_STARTED -> {
                // Being logged in on the splash screen and not having started onboarding means this
                // is not a new installation of the app, so skip onboarding all together.
                preferencesRepository.setOnboardingCheckpoint(OnboardingCheckpoint.COMPLETED)
                getLoggedInStartDestination()
            }

            OnboardingCheckpoint.WELCOME -> SplashUiSideEffect.NavigateToWelcome

            OnboardingCheckpoint.NOTIFICATIONS ->
                SplashUiSideEffect.NavigateToOnboarding(OnboardingStep.Notifications)

            OnboardingCheckpoint.GOOD_TO_KNOW ->
                SplashUiSideEffect.NavigateToOnboarding(GoodToKnowStep.GoodToKnow1)

            else -> getLoggedInStartDestination()
        }

    // We know the user is logged in, but we need to find out if their account has expired
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

    data object NavigateToWelcome : SplashUiSideEffect

    data class NavigateToOnboarding(val step: OnboardingStep) : SplashUiSideEffect
}
