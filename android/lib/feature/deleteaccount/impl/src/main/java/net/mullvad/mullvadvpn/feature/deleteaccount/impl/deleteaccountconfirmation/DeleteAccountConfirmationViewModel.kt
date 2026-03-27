package net.mullvad.mullvadvpn.feature.deleteaccount.impl.deleteaccountconfirmation

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.TimeoutCancellationException
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeout
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.util.daysLeft
import net.mullvad.mullvadvpn.lib.common.util.delayAtLeast
import net.mullvad.mullvadvpn.lib.model.DeleteAccountError
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.lib.repository.DeviceRepository

class DeleteAccountConfirmationViewModel(
    private val deviceRepository: DeviceRepository,
    private val accountRepository: AccountRepository,
) : ViewModel() {

    private val _uiSideEffect = Channel<DeleteAccountConfirmationUiSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()
    private val accountInput = MutableStateFlow("")
    private val verificationError = MutableStateFlow<VerifyAccountError?>(null)
    private val isLoading = MutableStateFlow(false)

    val uiState: StateFlow<DeleteAccountConfirmationUiState> =
        combine(timeLeftFlow(), isLoading, verificationError) { timeLeft, isLoading, error ->
                DeleteAccountConfirmationUiState(
                    isLoading = isLoading,
                    verificationError = error,
                    daysLeft = timeLeft,
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                DeleteAccountConfirmationUiState(
                    isLoading = false,
                    verificationError = null,
                    daysLeft = DaysLeftState.Loading,
                ),
            )

    private fun timeLeftFlow(): Flow<DaysLeftState> = flow {
        emit(DaysLeftState.Loading)

        try {
            val accountExpiry =
                withTimeout(3.seconds) { accountRepository.accountData.filterNotNull().first() }
            val daysLeft = accountExpiry.expiryDate.daysLeft()?.toInt()
            if (daysLeft == null) {
                emit(DaysLeftState.NoDaysLeft)
            } else {
                emit(DaysLeftState.DaysLeft(daysLeft))
            }
        } catch (_: TimeoutCancellationException) {
            emit(DaysLeftState.Error)
        }
    }

    fun deleteAccount() = viewModelScope.launch {
        isLoading.value = true

        val accountNumber = deviceRepository.deviceState.value?.accountNumber()
        if (accountNumber == null) {
            verificationError.value = VerifyAccountError.CouldNotFetchAccountNumber
            isLoading.value = false
            return@launch
        }

        if (accountInput.value != accountNumber.value) {
            verificationError.value = VerifyAccountError.AccountDoesNotMatch
            isLoading.value = false
            return@launch
        }

        delayAtLeast(1.seconds) { accountRepository.deleteAccount() }
            .fold(
                {
                    _uiSideEffect.send(
                        DeleteAccountConfirmationUiSideEffect.DeleteAccountFailed(it)
                    )
                    isLoading.value = false
                },
                { _uiSideEffect.send(DeleteAccountConfirmationUiSideEffect.NavigateToComplete) },
            )
    }

    fun onAccountInputChanged(input: String) {
        verificationError.value = null
        accountInput.value = input
    }
}

data class DeleteAccountConfirmationUiState(
    val isLoading: Boolean = false,
    val verificationError: VerifyAccountError? = null,
    val daysLeft: DaysLeftState,
)

sealed interface DeleteAccountConfirmationUiSideEffect {
    object NavigateToComplete : DeleteAccountConfirmationUiSideEffect

    data class DeleteAccountFailed(val deleteAccountError: DeleteAccountError) :
        DeleteAccountConfirmationUiSideEffect
}

sealed interface VerifyAccountError {
    object CouldNotFetchAccountNumber : VerifyAccountError

    object AccountDoesNotMatch : VerifyAccountError
}

sealed interface DaysLeftState {
    data object Loading : DaysLeftState

    data object Error : DaysLeftState

    data class DaysLeft(val value: Int) : DaysLeftState

    object NoDaysLeft : DaysLeftState
}
