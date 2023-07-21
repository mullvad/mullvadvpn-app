package net.mullvad.mullvadvpn.ui.fragment

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.AccountScreen
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.ui.NavigationBarPainter
import net.mullvad.mullvadvpn.ui.StatusBarPainter
import net.mullvad.mullvadvpn.viewmodel.AccountViewModel
import org.koin.androidx.viewmodel.ext.android.viewModel

class AccountFragment : BaseFragment(), StatusBarPainter, NavigationBarPainter {
    private val vm by viewModel<AccountViewModel>()

    @OptIn(ExperimentalMaterial3Api::class)
    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        return inflater.inflate(R.layout.fragment_compose, container, false).apply {
            findViewById<ComposeView>(R.id.compose_view).setContent {
                AppTheme {
                    val state = vm.uiState.collectAsState().value
                    AccountScreen(
                        uiState = state,
                        onManageAccountClick = vm::onManageAccountClick,
                        onRedeemVoucherClick = { openRedeemVoucherFragment() },
                        onLogoutClick = vm::onLogoutClick
                    ) {
                        activity?.onBackPressed()
                    }
                }
            }
        }
    }

    private fun openRedeemVoucherFragment() {
        val transaction = parentFragmentManager.beginTransaction()
        transaction.addToBackStack(null)
        RedeemVoucherDialogFragment().show(transaction, null)
    }

    /*
    // Injected dependencies
    private val accountRepository: AccountRepository by inject()
    private val deviceRepository: DeviceRepository by inject()
    private val serviceConnectionManager: ServiceConnectionManager by inject()

    private val dateStyle = DateFormat.MEDIUM
    private val timeStyle = DateFormat.SHORT
    private val expiryFormatter = DateFormat.getDateTimeInstance(dateStyle, timeStyle)

    private var hasConnectivity = true
        set(value) {
            field = value
            accountManagementButton.isEnabled = value
        }

    private var isOffline = true
        set(value) {
            field = value
            redeemVoucherButton.setEnabled(!value)
        }

    private var isAccountNumberShown by
        observable(false) { _, _, doShow ->
            accountNumberView.informationState =
                if (doShow) {
                    InformationView.Masking.Show(GroupedTransformationMethod())
                } else {
                    InformationView.Masking.Hide(GroupedPasswordTransformationMethod())
                }
        }
    private lateinit var accountExpiryView: InformationView
    private lateinit var accountNumberView: CopyableInformationView
    private lateinit var deviceNameView: InformationView
    private lateinit var accountManagementButton: AccountManagementButton
    private lateinit var redeemVoucherButton: RedeemVoucherButton
    private lateinit var titleController: CollapsibleTitleController

    @Deprecated("Refactor code to instead rely on Lifecycle.") private val jobTracker = JobTracker()

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

        view.findViewById<View>(R.id.close).setOnClickListener {
            requireMainActivity().onBackPressed()
        }

        accountManagementButton =
            view.findViewById<AccountManagementButton>(R.id.account_management).apply {
                setOnClickAction("openAccountPageInBrowser", jobTracker) {
                    isEnabled = false
                    serviceConnectionManager.authTokenCache()?.fetchAuthToken()?.let { token ->
                        context.openAccountPageInBrowser(token)
                    }
                    isEnabled = true
                    checkForAddedTime()
                }
            }
        accountManagementButton.isVisible = BuildTypes.RELEASE != BuildConfig.BUILD_TYPE

        redeemVoucherButton =
            view.findViewById<RedeemVoucherButton>(R.id.redeem_voucher).apply {
                prepare(parentFragmentManager, jobTracker)
            }

        view.findViewById<Button>(R.id.logout).setOnClickAction("logout", jobTracker) {
            accountRepository.logout()
        }

        accountNumberView =
            view.findViewById<CopyableInformationView>(R.id.account_number).apply {
                informationState =
                    InformationView.Masking.Hide(GroupedPasswordTransformationMethod())
                onToggleMaskingClicked = { isAccountNumberShown = isAccountNumberShown.not() }
            }

        accountExpiryView = view.findViewById(R.id.account_expiry)
        deviceNameView = view.findViewById(R.id.device_name)
        titleController = CollapsibleTitleController(view)

        return view
    }

    override fun onResume() {
        super.onResume()
        paintNavigationBar(ContextCompat.getColor(requireContext(), R.color.darkBlue))
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
            launchPaintStatusBarAfterTransition()
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
                    deviceNameView.information = state.deviceName()?.capitalizeFirstCharOfEachWord()
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

                        callbackFlowFromNotifier(state.container.connectionProxy.onUiStateChange)
                    } else {
                        emptyFlow()
                    }
                }
                .collect { uiState ->
                    hasConnectivity =
                        uiState is TunnelState.Connected ||
                            uiState is TunnelState.Disconnected ||
                            (uiState is TunnelState.Error && !uiState.errorState.isBlocking)
                    isOffline =
                        uiState is TunnelState.Error &&
                            uiState.errorState.cause is ErrorStateCause.IsOffline
                }
        }
    }

    private fun CoroutineScope.launchPaintStatusBarAfterTransition() = launch {
        transitionFinishedFlow.collect {
            paintStatusBar(ContextCompat.getColor(requireContext(), R.color.darkBlue))
        }
    }

    private fun CoroutineScope.launchRefreshDeviceStateAfterAnimation() = launch {
        transitionFinishedFlow.collect { deviceRepository.refreshDeviceState() }
    }

    private fun updateAccountExpiry(accountExpiry: DateTime?) {
        if (accountExpiry != null) {
            accountExpiryView.information = expiryFormatter.format(accountExpiry.toDate())
        } else {
            accountExpiryView.information = null
            accountRepository.fetchAccountExpiry()
        }
    }
    // */
}
