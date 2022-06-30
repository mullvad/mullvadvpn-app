package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.flowWithLifecycle
import androidx.lifecycle.lifecycleScope
import java.text.DateFormat
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.serviceconnection.AccountCache
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
    private val accountCache: AccountCache by inject()
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

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        lifecycleScope.launch {
            deviceRepository.deviceState
                .flowWithLifecycle(lifecycle, Lifecycle.State.RESUMED)
                .collect { state ->
                    accountNumberView.information = state.token()
                    deviceNameView.information = state.deviceName()?.capitalizeFirstCharOfEachWord()
                }
        }
    }

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
            accountCache.logout()
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
        jobTracker.newUiJob("updateAccountExpiry") {
            accountCache.accountExpiryState
                .map { state -> state.date() }
                .collect { expiryDate ->
                    currentAccountExpiry = expiryDate
                    updateAccountExpiry(expiryDate)
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

        sitePaymentButton.updateAuthTokenCache(authTokenCache)
        accountCache.fetchAccountExpiry()
    }

    override fun onSafelyStop() {
        jobTracker.cancelAllJobs()
    }

    override fun onSafelyDestroyView() {
        titleController.onDestroy()
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
            accountCache.fetchAccountExpiry()
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
