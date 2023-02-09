package net.mullvad.mullvadvpn.ui.fragment

import android.app.Activity
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import java.text.DateFormat
import kotlin.properties.Delegates.observable
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.CollapsibleTitleController
import net.mullvad.mullvadvpn.ui.GroupedPasswordTransformationMethod
import net.mullvad.mullvadvpn.ui.GroupedTransformationMethod
import net.mullvad.mullvadvpn.ui.extension.openAccountPageInBrowser
import net.mullvad.mullvadvpn.ui.extension.requireMainActivity
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.authTokenCache
import net.mullvad.mullvadvpn.ui.widget.Button
import net.mullvad.mullvadvpn.ui.widget.CopyableInformationView
import net.mullvad.mullvadvpn.ui.widget.InformationView
import net.mullvad.mullvadvpn.ui.widget.RedeemVoucherButton
import net.mullvad.mullvadvpn.ui.widget.SitePaymentButton
import net.mullvad.mullvadvpn.util.JobTracker
import net.mullvad.mullvadvpn.util.UNKNOWN_STATE_DEBOUNCE_DELAY_MILLISECONDS
import net.mullvad.mullvadvpn.util.addDebounceForUnknownState
import net.mullvad.mullvadvpn.util.callbackFlowFromNotifier
import net.mullvad.mullvadvpn.util.capitalizeFirstCharOfEachWord
import net.mullvad.talpid.tunnel.ErrorStateCause
import org.joda.time.DateTime
import org.koin.android.ext.android.inject

class AccountFragment : BaseFragment() {

    // Injected dependencies
    private val accountRepository: AccountRepository by inject()
    private val deviceRepository: DeviceRepository by inject()
    private val serviceConnectionManager: ServiceConnectionManager by inject()

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

    private var isAccountNumberShown by observable(false) { _, _, doShow ->
        accountNumberView.informationState = if (doShow) {
            InformationView.Masking.Show(GroupedTransformationMethod())
        } else {
            InformationView.Masking.Hide(GroupedPasswordTransformationMethod())
        }
    }

    private lateinit var accountExpiryView: InformationView
    private lateinit var accountNumberView: CopyableInformationView
    private lateinit var deviceNameView: InformationView
    private lateinit var sitePaymentButton: SitePaymentButton
    private lateinit var redeemVoucherButton: RedeemVoucherButton
    private lateinit var titleController: CollapsibleTitleController

    @Deprecated("Refactor code to instead rely on Lifecycle.")
    private val jobTracker = JobTracker()

    override fun onAttach(activity: Activity) {
        super.onAttach(activity)
        requireMainActivity().enterSecureScreen(this)
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        lifecycleScope.launchUiSubscriptionsOnResume()
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.account, container, false)

        view.findViewById<View>(R.id.back).setOnClickListener {
            requireMainActivity().onBackPressed()
        }

        sitePaymentButton = view.findViewById<SitePaymentButton>(R.id.site_payment).apply {
            newAccount = false

            setOnClickAction("openAccountPageInBrowser", jobTracker) {
                setEnabled(false)
                serviceConnectionManager.authTokenCache()?.fetchAuthToken()?.let { token ->
                    context.openAccountPageInBrowser(token)
                }
                setEnabled(true)
                checkForAddedTime()
            }
        }

        redeemVoucherButton = view.findViewById<RedeemVoucherButton>(R.id.redeem_voucher).apply {
            prepare(parentFragmentManager, jobTracker)
        }

        view.findViewById<Button>(R.id.logout).setOnClickAction("logout", jobTracker) {
            accountRepository.logout()
        }

        accountNumberView = view.findViewById<CopyableInformationView>(R.id.account_number).apply {
            informationState = InformationView.Masking.Hide(GroupedPasswordTransformationMethod())
            onToggleMaskingClicked = {
                isAccountNumberShown = isAccountNumberShown.not()
            }
        }

        accountExpiryView = view.findViewById(R.id.account_expiry)
        deviceNameView = view.findViewById(R.id.device_name)
        titleController = CollapsibleTitleController(view)

        return view
    }

    override fun onStop() {
        jobTracker.cancelAllJobs()
        super.onStop()
    }

    override fun onDestroyView() {
        titleController.onDestroy()
        super.onDestroyView()
    }

    override fun onDetach() {
        requireMainActivity().leaveSecureScreen(this)
        super.onDetach()
    }

    private fun CoroutineScope.launchUiSubscriptionsOnResume() = launch {
        repeatOnLifecycle(Lifecycle.State.RESUMED) {
            launchUpdateTextOnDeviceChanges()
            launchUpdateTextOnExpiryChanges()
            launchTunnelStateSubscription()
            launchRefreshDeviceStateAfterAnimation()
        }
    }

    private fun CoroutineScope.launchUpdateTextOnDeviceChanges() {
        launch {
            deviceRepository.deviceState
                .debounce {
                    it.addDebounceForUnknownState(UNKNOWN_STATE_DEBOUNCE_DELAY_MILLISECONDS)
                }
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
                .map { state -> state.date() }
                .collect { expiryDate ->
                    currentAccountExpiry = expiryDate
                    updateAccountExpiry(expiryDate)
                }
        }
    }

    private fun CoroutineScope.launchTunnelStateSubscription() {
        launch {
            serviceConnectionManager.connectionState
                .flatMapLatest { state ->
                    if (state is ServiceConnectionState.ConnectedReady) {
                        callbackFlowFromNotifier(
                            state.container.connectionProxy.onUiStateChange
                        )
                    } else {
                        emptyFlow()
                    }
                }
                .collect { uiState ->
                    hasConnectivity = uiState is TunnelState.Connected ||
                        uiState is TunnelState.Disconnected ||
                        (uiState is TunnelState.Error && !uiState.errorState.isBlocking)
                    isOffline = uiState is TunnelState.Error &&
                        uiState.errorState.cause is ErrorStateCause.IsOffline
                }
        }
    }

    private fun CoroutineScope.launchRefreshDeviceStateAfterAnimation() = launch {
        transitionFinishedFlow.collect {
            deviceRepository.refreshDeviceState()
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
}
