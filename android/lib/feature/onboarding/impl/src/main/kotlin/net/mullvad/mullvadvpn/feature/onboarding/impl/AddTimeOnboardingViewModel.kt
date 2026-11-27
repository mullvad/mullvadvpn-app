package net.mullvad.mullvadvpn.feature.onboarding.impl

import androidx.lifecycle.Lifecycle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.repeatOnLifecycle
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.util.ACCOUNT_EXPIRY_POLL_INTERVAL
import net.mullvad.mullvadvpn.lib.common.util.isAfterNowInstant
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.payment.util.isSuccess
import net.mullvad.mullvadvpn.lib.payment.util.status
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.lib.repository.PaymentLogic
import net.mullvad.mullvadvpn.lib.repository.PlayPaymentLogic.Companion.VERIFICATION_POLL_INTERVAL
import net.mullvad.mullvadvpn.lib.userpreferences.OnboardingCheckpoint
import net.mullvad.mullvadvpn.lib.userpreferences.UserPreferencesRepository

data class AddTimeOnboardingUiState(val paymentStatus: PaymentStatus? = null)

class AddTimeOnboardingViewModel(
    private val accountRepository: AccountRepository,
    private val paymentUseCase: PaymentLogic,
    userPreferencesRepository: UserPreferencesRepository,
    activityLifecycle: Lifecycle,
) : ViewModel() {

    val uiState: StateFlow<AddTimeOnboardingUiState> =
        paymentUseCase.paymentAvailability
            .map { AddTimeOnboardingUiState(paymentStatus = it?.status()) }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                AddTimeOnboardingUiState(),
            )

    val uiSideEffect: SharedFlow<UiSideEffect> =
        combine(
                accountRepository.accountData,
                userPreferencesRepository.onboardingCheckpoint(),
                ::AccountDataEvent,
            )
            .filter { hasAddedTime(it.accountData) }
            .map { event ->
                UiSideEffect.TimeAdded(
                    hasCompletedOnboarding =
                        event.currentCheckpoint == OnboardingCheckpoint.COMPLETED,
                )
            }
            .onEach {
                paymentUseCase.resetPurchaseResult()
                accountRepository.resetIsNewAccount()
            }
            .shareIn(
                viewModelScope,
                started = SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
            )

    init {
        viewModelScope.launch {
            activityLifecycle.repeatOnLifecycle(Lifecycle.State.STARTED) { pollAccountExpiry() }
        }
        viewModelScope.launch {
            activityLifecycle.repeatOnLifecycle(Lifecycle.State.STARTED) {
                pollPurchaseVerification()
            }
        }
    }

    private fun hasAddedTime(accountData: AccountData?): Boolean =
        accountData?.expiryDate?.minusHours(MIN_HOURS_PAST_ACCOUNT_EXPIRY)?.isAfterNowInstant()
            ?: false

    private suspend fun updateAccountExpiry() {
        accountRepository.fetchAccountData()
    }

    private suspend fun pollAccountExpiry() {
        while (currentCoroutineContext().isActive) {
            updateAccountExpiry()
            delay(ACCOUNT_EXPIRY_POLL_INTERVAL)
        }
    }

    private suspend fun pollPurchaseVerification() {
        while (currentCoroutineContext().isActive) {
            // We do not want to retry verification if it fails, since we are already polling
            // for it.
            if (paymentUseCase.verifyPurchases(maxAttempts = 0).isSuccess()) {
                updateAccountExpiry()
            }
            delay(VERIFICATION_POLL_INTERVAL)
        }
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
    val currentCheckpoint: OnboardingCheckpoint,
)
