package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import java.text.DateFormat
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.extension.openAccountPageInBrowser
import net.mullvad.mullvadvpn.ui.serviceconnection.AccountRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.DeviceRepository
import net.mullvad.mullvadvpn.ui.widget.Button
import net.mullvad.mullvadvpn.ui.widget.CopyableInformationView
import net.mullvad.mullvadvpn.ui.widget.InformationView
import net.mullvad.mullvadvpn.ui.widget.RedeemVoucherButton
import net.mullvad.mullvadvpn.ui.widget.SitePaymentButton
import net.mullvad.mullvadvpn.util.capitalizeFirstCharOfEachWord
import net.mullvad.talpid.tunnel.ErrorStateCause
import org.joda.time.DateTime
import org.koin.android.ext.android.inject

class AccountFragment : ServiceDependentFragment(OnNoService.GoBack) {

    // Injected dependencies
    private val accountRepository: AccountRepository by inject()
    private val deviceRepository: DeviceRepository by inject()

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

            setOnClickAction("openAccountPageInBrowser", jobTracker) {
                setEnabled(false)
                context.openAccountPageInBrowser(authTokenCache.fetchAuthToken())
                setEnabled(true)
            }
        }

        redeemVoucherButton = view.findViewById<RedeemVoucherButton>(R.id.redeem_voucher).apply {
            prepare(parentFragmentManager, jobTracker)
        }

        view.findViewById<Button>(R.id.logout).setOnClickAction("logout", jobTracker) {
            accountRepository.logout()
        }

        accountNumberView = view.findViewById<CopyableInformationView>(R.id.account_number).apply {
            displayFormatter = { rawAccountNumber -> addSpacesToAccountNumber(rawAccountNumber) }
        }

        accountExpiryView = view.findViewById(R.id.account_expiry)
        deviceNameView = view.findViewById(R.id.device_name)
        titleController = CollapsibleTitleController(view)

        return view
    }

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        lifecycleScope.launchUiSubscriptionsOnResume()
    }

    override fun onSafelyStart() {
        connectionProxy.onUiStateChange.subscribe(this) { uiState ->
            jobTracker.newUiJob("updateHasConnectivity") {
                hasConnectivity = uiState is TunnelState.Connected ||
                    uiState is TunnelState.Disconnected ||
                    (uiState is TunnelState.Error && !uiState.errorState.isBlocking)
                isOffline = uiState is TunnelState.Error &&
                    uiState.errorState.cause is ErrorStateCause.IsOffline
            }
        }
    }

    override fun onSafelyStop() {
        jobTracker.cancelAllJobs()
    }

    override fun onSafelyDestroyView() {
        titleController.onDestroy()
    }

    private fun CoroutineScope.launchUiSubscriptionsOnResume() = launch {
        repeatOnLifecycle(Lifecycle.State.RESUMED) {
            launchUpdateTextOnDeviceChanges()
            launchUpdateTextOnExpiryChanges()
        }
    }

    private fun CoroutineScope.launchUpdateTextOnDeviceChanges() {
        launch {
            deviceRepository.deviceState
                .collect { state ->
                    accountNumberView.information = state.token()
                    deviceNameView.information =
                        state.deviceName()?.capitalizeFirstCharOfEachWord()
                }
        }
    }

    private fun CoroutineScope.launchUpdateTextOnExpiryChanges() {
        launch {
            accountRepository.accountExpiryState
                .onStart { accountRepository.fetchAccountExpiry() }
                .map { state -> state.date() }
                .collect { expiryDate ->
                    currentAccountExpiry = expiryDate
                    updateAccountExpiry(expiryDate)
                }
        }
    }

    private fun checkForAddedTime() {
        currentAccountExpiry?.let { expiry ->
            oldAccountExpiry = expiry
        }
    }

    private fun updateAccountExpiry(accountExpiry: DateTime?) {
        if (accountExpiry != null) {
            accountExpiryView.information = expiryFormatter.format(accountExpiry.toDate())
        } else {
            accountExpiryView.information = null
            accountRepository.fetchAccountExpiry()
        }
    }

    private fun showRedeemVoucherDialog() {
        val transaction = parentFragmentManager.beginTransaction()

        transaction.addToBackStack(null)

        RedeemVoucherDialogFragment().show(transaction, null)
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
