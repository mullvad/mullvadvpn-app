package net.mullvad.mullvadvpn.feature.onboarding.impl

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.feature.onboarding.api.GoodToKnowStep
import net.mullvad.mullvadvpn.feature.onboarding.api.OnboardingStep
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.util.isAfterNowInstant
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.lib.repository.PaymentLogic
import net.mullvad.mullvadvpn.lib.usecase.SelectedLocationTitleUseCase
import net.mullvad.mullvadvpn.lib.userpreferences.OnboardingCheckpoint
import net.mullvad.mullvadvpn.lib.userpreferences.UserPreferencesRepository

class OnboardingViewModel(
    private val userPreferencesRepository: UserPreferencesRepository,
    paymentUseCase: PaymentLogic,
    accountRepository: AccountRepository,
    selectedLocationTitleUseCase: SelectedLocationTitleUseCase,
) : ViewModel() {

    private val currentStep = MutableStateFlow<OnboardingStep?>(null)

    val uiState: StateFlow<OnboardingUiState> =
        selectedLocationTitleUseCase()
            .map(::OnboardingUiState)
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                OnboardingUiState(),
            )

    val uiSideEffect: Flow<UiSideEffect> =
        combine(
                accountRepository.accountData,
                currentStep,
                userPreferencesRepository.onboardingCheckpoint(),
                transform = ::AccountDataEvent,
            )
            .filter { (accountData, step) ->
                step == OnboardingStep.AddTime &&
                    accountData
                        ?.expiryDate
                        ?.minusHours(MIN_HOURS_PAST_ACCOUNT_EXPIRY)
                        ?.isAfterNowInstant() ?: false
            }
            .onEach {
                paymentUseCase.resetPurchaseResult()
                accountRepository.resetIsNewAccount()
            }
            .map {
                UiSideEffect.TimeAdded(
                    hasCompletedOnboarding = it.currentCheckpoint == OnboardingCheckpoint.COMPLETED,
                )
            }

    fun onStepEntered(step: OnboardingStep) {
        currentStep.value = step
        viewModelScope.launch {
            when (step) {
                OnboardingStep.Notifications ->
                    userPreferencesRepository.setOnboardingCheckpoint(
                        OnboardingCheckpoint.NOTIFICATIONS,
                    )

                GoodToKnowStep.GoodToKnow1 ->
                    userPreferencesRepository.setOnboardingCheckpoint(
                        OnboardingCheckpoint.GOOD_TO_KNOW,
                    )

                else -> Unit
            }
        }
    }

    fun onNotificationStepDismissed() = viewModelScope.launch {
        userPreferencesRepository.setNotificationPermissionAsked()
    }

    fun complete() = viewModelScope.launch {
        userPreferencesRepository.setOnboardingCheckpoint(
            OnboardingCheckpoint.PART_1_COMPLETED,
        )
    }

    sealed interface UiSideEffect {
        data class TimeAdded(val hasCompletedOnboarding: Boolean) : UiSideEffect
    }

    companion object {
        private const val MIN_HOURS_PAST_ACCOUNT_EXPIRY: Long = 20
    }
}

private data class AccountDataEvent(
    val accountData: AccountData?,
    val step: OnboardingStep?,
    val currentCheckpoint: OnboardingCheckpoint,
)
