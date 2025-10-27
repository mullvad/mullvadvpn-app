package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import java.time.ZonedDateTime
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.WebsiteAuthToken
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.lib.repository.DeviceRepository
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.hasPendingPayment
import net.mullvad.mullvadvpn.util.isSuccess
import net.mullvad.mullvadvpn.util.toLc

class AccountViewModel(
    private val accountRepository: AccountRepository,
    deviceRepository: DeviceRepository,
    private val paymentUseCase: PaymentUseCase,
) : ViewModel() {
    private val _uiSideEffect = Channel<UiSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val isLoggingOut = MutableStateFlow(false)

    val uiState: StateFlow<Lc<Unit, AccountUiState>> =
        combine(
                deviceRepository.deviceState.filterIsInstance<DeviceState.LoggedIn>(),
                accountData(),
                paymentUseCase.paymentAvailability,
                isLoggingOut,
            ) { deviceState, accountData, paymentAvailability, isLoggingOut ->
                AccountUiState(
                        deviceName = deviceState.device.displayName(),
                        accountNumber = deviceState.accountNumber,
                        accountExpiry = accountData?.expiryDate,
                        showLogoutLoading = isLoggingOut,
                        verificationPending = paymentAvailability.hasPendingPayment(),
                    )
                    .toLc<Unit, AccountUiState>()
            }
            .onStart { viewModelScope.launch { updateAccountExpiry() } }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(Unit),
            )

    init {
        verifyPurchases()
    }

    private fun accountData(): Flow<AccountData?> =
        // Ignore nulls expect first, to avoid loading when logging out.
        accountRepository.accountData
            .filterNotNull()
            .onStart<AccountData?> { emit(accountRepository.accountData.value) }
            .distinctUntilChanged()

    fun onLogoutClick() {
        if (isLoggingOut.value) return
        isLoggingOut.value = true

        viewModelScope.launch {
            accountRepository
                .logout()
                .also { isLoggingOut.value = false }
                .fold(
                    { _uiSideEffect.send(UiSideEffect.GenericError) },
                    { _uiSideEffect.send(UiSideEffect.NavigateToLogin) },
                )
        }
    }

    fun onCopyAccountNumber(accountNumber: String) {
        viewModelScope.launch { _uiSideEffect.send(UiSideEffect.CopyAccountNumber(accountNumber)) }
    }

    private fun updateAccountExpiry() {
        viewModelScope.launch { accountRepository.refreshAccountData() }
    }

    private fun verifyPurchases() {
        viewModelScope.launch {
            if (paymentUseCase.verifyPurchases().isSuccess()) {
                updateAccountExpiry()
            }
        }
    }

    sealed class UiSideEffect {
        data object NavigateToLogin : UiSideEffect()

        data class OpenAccountManagementPageInBrowser(val token: WebsiteAuthToken?) :
            UiSideEffect()

        data class CopyAccountNumber(val accountNumber: String) : UiSideEffect()

        data object GenericError : UiSideEffect()
    }
}

data class AccountUiState(
    val deviceName: String,
    val accountNumber: AccountNumber,
    val accountExpiry: ZonedDateTime?,
    val showLogoutLoading: Boolean,
    val verificationPending: Boolean,
)
