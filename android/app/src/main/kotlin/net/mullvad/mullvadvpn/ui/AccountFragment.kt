package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.fragment.app.FragmentManager
import java.text.DateFormat
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.onEach
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.widget.Button
import net.mullvad.mullvadvpn.ui.widget.CopyableInformationView
import net.mullvad.mullvadvpn.ui.widget.InformationView
import net.mullvad.mullvadvpn.ui.widget.RedeemVoucherButton
import net.mullvad.mullvadvpn.ui.widget.SitePaymentButton
import net.mullvad.mullvadvpn.util.capitalizeFirstCharOfEachWord
import net.mullvad.talpid.tunnel.ErrorStateCause
import org.joda.time.DateTime

class AccountFragment : ServiceDependentFragment(OnNoService.GoBack) {
    override val isSecureScreen = true

    private val dateStyle = DateFormat.MEDIUM
    private val timeStyle = DateFormat.SHORT
    private val expiryFormatter = DateFormat.getDateTimeInstance(dateStyle, timeStyle)

    private var oldAccountExpiry: DateTime? = null

    private var currentAccountExpiry: DateTime? = null
        set(value) {
            field = value

            synchronized(this) {
                if (value != oldAccountExpiry) {
                    oldAccountExpiry = null
                }
            }
        }

    private var hasConnectivity = true
        set(value) {
            field = value
            sitePaymentButton.setEnabled(value)
        }

    private var isOffline = true
        set(value) {
            field = value
            redeemVoucherButton.setEnabled(!value)
        }

    private lateinit var accountExpiryView: InformationView
    private lateinit var accountNumberView: CopyableInformationView
    private lateinit var deviceNameView: InformationView
    private lateinit var sitePaymentButton: SitePaymentButton
    private lateinit var redeemVoucherButton: RedeemVoucherButton
    private lateinit var titleController: CollapsibleTitleController

    override fun onSafelyCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.account, container, false)

        view.findViewById<View>(R.id.back).setOnClickListener {
            parentActivity.onBackPressed()
        }

        sitePaymentButton = view.findViewById<SitePaymentButton>(R.id.site_payment).apply {
            newAccount = false

            prepare(authTokenCache, jobTracker) {
                checkForAddedTime()
            }
        }

        redeemVoucherButton = view.findViewById<RedeemVoucherButton>(R.id.redeem_voucher).apply {
            prepare(parentFragmentManager, jobTracker)
        }

        view.findViewById<Button>(R.id.logout).setOnClickAction("logout", jobTracker) {
            logout()
        }

        accountNumberView = view.findViewById<CopyableInformationView>(R.id.account_number).apply {
            displayFormatter = { rawAccountNumber -> addSpacesToAccountNumber(rawAccountNumber) }
        }

        accountExpiryView = view.findViewById(R.id.account_expiry)
        deviceNameView = view.findViewById(R.id.device_name)
        titleController = CollapsibleTitleController(view)

        return view
    }

    override fun onSafelyStart() {
        jobTracker.newUiJob("updateAccountNumber") {
            deviceRepository.deviceState
                .onEach { state ->
                    if (state.isInitialState()) deviceRepository.refreshDeviceState()
                }
                .collect { state ->
                    accountNumberView.information = state.token()
                }
        }

        jobTracker.newUiJob("updateDeviceName") {
            deviceRepository.deviceState
                .onEach { state ->
                    if (state.isInitialState()) deviceRepository.refreshDeviceState()
                }
                .collect { state ->
                    deviceNameView.information = state.deviceName()?.capitalizeFirstCharOfEachWord()
                }
        }

        accountCache.onAccountExpiryChange.subscribe(this) { accountExpiry ->
            jobTracker.newUiJob("updateAccountExpiry") {
                currentAccountExpiry = accountExpiry
                updateAccountExpiry(accountExpiry)
            }
        }

        connectionProxy.onUiStateChange.subscribe(this) { uiState ->
            jobTracker.newUiJob("updateHasConnectivity") {
                hasConnectivity = uiState is TunnelState.Connected ||
                    uiState is TunnelState.Disconnected ||
                    (uiState is TunnelState.Error && !uiState.errorState.isBlocking)
                isOffline = uiState is TunnelState.Error &&
                    uiState.errorState.cause is ErrorStateCause.IsOffline
            }
        }

        oldAccountExpiry?.let { expiry ->
            accountCache.invalidateAccountExpiry(expiry)
        }

        sitePaymentButton.updateAuthTokenCache(authTokenCache)
    }

    override fun onSafelyStop() {
        accountCache.onAccountExpiryChange.unsubscribe(this)
        jobTracker.cancelAllJobs()
    }

    override fun onSafelyDestroyView() {
        titleController.onDestroy()
    }

    private fun checkForAddedTime() {
        currentAccountExpiry?.let { expiry ->
            oldAccountExpiry = expiry
            accountCache.invalidateAccountExpiry(expiry)
        }
    }

    private fun updateAccountExpiry(accountExpiry: DateTime?) {
        if (accountExpiry != null) {
            accountExpiryView.information = expiryFormatter.format(accountExpiry.toDate())
        } else {
            accountExpiryView.information = null
            accountCache.fetchAccountExpiry()
        }
    }

    private fun showRedeemVoucherDialog() {
        val transaction = parentFragmentManager.beginTransaction()

        transaction.addToBackStack(null)

        RedeemVoucherDialogFragment().show(transaction, null)
    }

    private suspend fun logout() {
        accountCache.logout()
        clearBackStack()
        goToLoginScreen()
    }

    private fun clearBackStack() {
        parentFragmentManager.apply {
            val firstEntry = getBackStackEntryAt(0)

            popBackStack(firstEntry.id, FragmentManager.POP_BACK_STACK_INCLUSIVE)
        }
    }

    private fun goToLoginScreen() {
        parentFragmentManager.beginTransaction().apply {
            setCustomAnimations(
                R.anim.do_nothing,
                R.anim.fragment_exit_to_bottom,
                R.anim.do_nothing,
                R.anim.do_nothing
            )
            replace(R.id.main_fragment, LoginFragment())
            commit()
        }
    }

    private fun addSpacesToAccountNumber(rawAccountNumber: String): String {
        return rawAccountNumber
            .asSequence()
            .fold(StringBuilder()) { formattedAccountNumber, nextDigit ->
                if ((formattedAccountNumber.length % 5) == 4) {
                    formattedAccountNumber.append(' ')
                }

                formattedAccountNumber.append(nextDigit)
            }
            .toString()
    }
}
