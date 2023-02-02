package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.material.ExperimentalMaterialApi
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import androidx.fragment.app.FragmentActivity
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import java.net.InetAddress
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.AdvancedSettingScreen
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.ui.customdns.CustomDnsAdapter
import net.mullvad.mullvadvpn.ui.extension.requireMainActivity
import net.mullvad.mullvadvpn.ui.fragment.BaseFragment
import net.mullvad.mullvadvpn.ui.fragment.ConfirmDnsDialogFragment
import net.mullvad.mullvadvpn.ui.fragment.SplitTunnelingFragment
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.customDns
import net.mullvad.mullvadvpn.ui.widget.MtuCell
import net.mullvad.mullvadvpn.ui.widget.ToggleCell
import net.mullvad.mullvadvpn.util.callbackFlowFromNotifier
import net.mullvad.mullvadvpn.viewmodel.AdvancedSettingViewModel
import org.koin.android.ext.android.inject
import org.koin.androidx.viewmodel.ext.android.viewModel

class AdvancedSettingFragment : BaseFragment() {

    // Injected dependencies
    private val serviceConnectionManager: ServiceConnectionManager by inject()

    private val advancesSettingViewModel by viewModel<AdvancedSettingViewModel>()

    private var isAllowLanEnabled = false

    // Both customDnsAdapter and customDnsToggle are nullable since onNewServiceConnection,
    // which sets up custom dns subscriptions, is called before onSafelyCreateView.
    private var customDnsAdapter: CustomDnsAdapter? = null
    private var customDnsToggle: ToggleCell? = null

    private lateinit var wireguardMtuInput: MtuCell

    val shared = serviceConnectionManager.connectionState
        .flatMapLatest { state ->
            if (state is ServiceConnectionState.ConnectedReady) {
                flowOf(state.container)
            } else {
                emptyFlow()
            }
        }
        .map {
            it.customDns
        }
        .shareIn(lifecycleScope, SharingStarted.WhileSubscribed())

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        lifecycleScope.launch {
            repeatOnLifecycle(Lifecycle.State.RESUMED) {
                launch {
                    serviceConnectionManager.connectionState
                        .flatMapLatest { state ->
                            if (state is ServiceConnectionState.ConnectedReady) {
                                flowOf(state.container)
                            } else {
                                emptyFlow()
                            }
                        }
                        .flatMapLatest {
                            callbackFlowFromNotifier(it.settingsListener.settingsNotifier)
                        }
                        .collect { settings ->
                            if (settings != null) {
                                updateUi(settings)
                            }
                        }
                }

                launch {
                    shared
                        .flatMapLatest {
                            callbackFlowFromNotifier(it.onEnabledChanged)
                        }
                        .collect { isEnabled ->
                            customDnsAdapter?.updateState(isEnabled)
                        }
                }

                launch {
                    shared
                        .flatMapLatest {
                            callbackFlowFromNotifier(it.onDnsServersChanged)
                        }
                        .collect { servers ->
                            customDnsAdapter?.updateServers(servers)
                        }
                }
            }
        }
    }

    override fun onDestroyView() {
        detachBackButtonHandler()
        customDnsAdapter?.onDestroy()
        super.onDestroyView()
    }

    @OptIn(ExperimentalMaterialApi::class)
    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {

        //
        return inflater.inflate(R.layout.fragment_compose, container, false).apply {
            findViewById<ComposeView>(R.id.compose_view).setContent {
                AdvancedSettingScreen(
                    uiState = advancesSettingViewModel.uiState.collectAsState().value,
                    onDnsCellClicked = { advancesSettingViewModel.setEditDnsIndex(it) },
                    onToggleCustomDns = { advancesSettingViewModel.toggleCustomDns(it) },
                    onNavigateCellClicked = { onNavigationCellClicked(requireActivity()) },
                    onAddDnsChanged = { item -> advancesSettingViewModel.addDnsClicked(item) },
                    onRemoveDnsChanged = { index ->
                        advancesSettingViewModel.removeDnsClicked(index)
                    },
                    onEditDnsChanged = { index, item ->
                        advancesSettingViewModel.editDnsClicked(
                            index,
                            item
                        )
                    },
                    onDnsChanged = { index, item ->
                        advancesSettingViewModel.dnsChanged(
                            index,
                            item
                        )
                    },
                    onBackClick = { },
                    onMtuChanged = { advancesSettingViewModel.onMtuChanged(it) },
                    onMtuSubmit = { advancesSettingViewModel.onSubmitMtu() },
                )
            }
            // setup adapter
            customDnsAdapter = CustomDnsAdapter(
                onAddServer = { address ->
                    serviceConnectionManager.customDns()?.addDnsServer(address) ?: false
                },
                onRemoveDnsServer = { address ->
                    serviceConnectionManager.customDns()?.removeDnsServer(address) ?: false
                },
                onSetCustomDnsEnabled = { isEnabled ->
                    if (isEnabled) {
                        serviceConnectionManager.customDns()?.enable()
                    } else {
                        serviceConnectionManager.customDns()?.disable()
                    }
                },
                onReplaceDnsServer = { oldServer, newServer ->
                    serviceConnectionManager.customDns()?.replaceDnsServer(
                        oldServer,
                        newServer
                    ) ?: false
                }
            ).also { newCustomDnsAdapter ->

                newCustomDnsAdapter.confirmAddAddress = ::confirmAddAddress
            }
            attachBackButtonHandler()
        }
    }

    private fun updateUi(settings: Settings) {
//        if (this::wireguardMtuInput.isInitialized && wireguardMtuInput.hasFocus == false) {
//            wireguardMtuInput.value = settings.tunnelOptions.wireguard.options.mtu
//        }
    }

    private suspend fun confirmAddAddress(address: InetAddress): Boolean {
        val isLocalAddress = address.isLinkLocalAddress() || address.isSiteLocalAddress()

        return !isLocalAddress || isAllowLanEnabled || showConfirmDnsServerDialog()
    }

    private suspend fun showConfirmDnsServerDialog(): Boolean {
        val confirmation = CompletableDeferred<Boolean>()
        val transaction = parentFragmentManager.beginTransaction()

        detachBackButtonHandler()
        transaction.addToBackStack(null)

        ConfirmDnsDialogFragment(confirmation)
            .show(transaction, null)

        val result = confirmation.await()

        attachBackButtonHandler()

        return result
    }

    private fun attachBackButtonHandler() {
        requireMainActivity().backButtonHandler = {
            if (customDnsAdapter?.isEditing == true) {
                customDnsAdapter?.stopEditing()
            }
            false
        }
    }

    private fun detachBackButtonHandler() {
        requireMainActivity().backButtonHandler = null
    }

    private fun onNavigationCellClicked(fragmentActivity: FragmentActivity) {
        val fragment =
            SplitTunnelingFragment::class.java.getConstructor()
                .newInstance()

        fragmentActivity.supportFragmentManager
            ?.beginTransaction()
            ?.apply {
                setCustomAnimations(
                    R.anim.fragment_enter_from_right,
                    R.anim.fragment_exit_to_left,
                    R.anim.fragment_half_enter_from_left,
                    R.anim.fragment_exit_to_right
                )
                replace(R.id.main_fragment, fragment)
                addToBackStack(null)
                commitAllowingStateLoss()
            }
    }
}
