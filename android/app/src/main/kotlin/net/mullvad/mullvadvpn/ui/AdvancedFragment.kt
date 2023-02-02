package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import androidx.recyclerview.widget.LinearLayoutManager
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
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.ui.customdns.CustomDnsAdapter
import net.mullvad.mullvadvpn.ui.extension.requireMainActivity
import net.mullvad.mullvadvpn.ui.fragment.BaseFragment
import net.mullvad.mullvadvpn.ui.fragment.ConfirmDnsDialogFragment
import net.mullvad.mullvadvpn.ui.fragment.SplitTunnelingFragment
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.customDns
import net.mullvad.mullvadvpn.ui.serviceconnection.settingsListener
import net.mullvad.mullvadvpn.ui.widget.CellSwitch
import net.mullvad.mullvadvpn.ui.widget.CustomRecyclerView
import net.mullvad.mullvadvpn.ui.widget.MtuCell
import net.mullvad.mullvadvpn.ui.widget.NavigateCell
import net.mullvad.mullvadvpn.ui.widget.ToggleCell
import net.mullvad.mullvadvpn.util.AdapterWithHeader
import net.mullvad.mullvadvpn.util.JobTracker
import net.mullvad.mullvadvpn.util.callbackFlowFromNotifier
import org.koin.android.ext.android.inject

// TODO: Move as part of refactoring to compose.
class AdvancedFragment : BaseFragment() {

    // Injected dependencies
    private val serviceConnectionManager: ServiceConnectionManager by inject()

    private var isAllowLanEnabled = false

    // Both customDnsAdapter and customDnsToggle are nullable since onNewServiceConnection,
    // which sets up custom dns subscriptions, is called before onSafelyCreateView.
    private var customDnsAdapter: CustomDnsAdapter? = null
    private var customDnsToggle: ToggleCell? = null

    private lateinit var wireguardMtuInput: MtuCell
    private lateinit var titleController: CollapsibleTitleController

    @Deprecated("Refactor code to instead rely on Lifecycle.")
    private val jobTracker = JobTracker()

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
                            jobTracker.newUiJob("updateEnabled") {
                                if (isEnabled) {
                                    customDnsToggle?.state = CellSwitch.State.ON
                                } else {
                                    customDnsToggle?.state = CellSwitch.State.OFF
                                }
                            }
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

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        val view = inflater.inflate(R.layout.advanced, container, false)

        view.findViewById<View>(R.id.back).setOnClickListener {
            customDnsAdapter?.stopEditing()
            requireActivity().onBackPressed()
        }

        titleController = CollapsibleTitleController(view, R.id.contents)

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

            view.findViewById<CustomRecyclerView>(R.id.contents).apply {
                layoutManager = LinearLayoutManager(requireContext())

                adapter = AdapterWithHeader(newCustomDnsAdapter, R.layout.advanced_header).apply {
                    onHeaderAvailable = { headerView ->
                        configureHeader(headerView)
                        titleController.expandedTitleView =
                            headerView.findViewById(R.id.expanded_title)
                    }
                }

                addItemDecoration(
                    ListItemDividerDecoration(
                        topOffset = resources.getDimensionPixelSize(R.dimen.list_item_divider)
                    )
                )
            }
        }

        attachBackButtonHandler()

        return view
    }

    override fun onDestroyView() {
        detachBackButtonHandler()
        customDnsAdapter?.onDestroy()
        titleController.onDestroy()
        super.onDestroyView()
    }

    private fun configureHeader(view: View) {
        wireguardMtuInput = view.findViewById<MtuCell>(R.id.wireguard_mtu).apply {
            onSubmit = { mtu ->
                serviceConnectionManager.settingsListener()?.wireguardMtu = mtu
            }
            value = serviceConnectionManager.settingsListener()?.let { settingsNotifier ->
                settingsNotifier.wireguardMtu
            }
        }

        view.findViewById<NavigateCell>(R.id.split_tunneling).apply {
            targetFragment = SplitTunnelingFragment::class
        }

        customDnsToggle = view.findViewById<ToggleCell>(R.id.enable_custom_dns).apply {
            state = serviceConnectionManager.customDns().let { customDns ->
                if (customDns?.isCustomDnsEnabled() == true) {
                    CellSwitch.State.ON
                } else {
                    CellSwitch.State.OFF
                }
            }

            listener = { state ->
                jobTracker.newBackgroundJob("toggleCustomDns") {
                    if (state == CellSwitch.State.ON) {
                        serviceConnectionManager.customDns()?.enable()
                    } else {
                        serviceConnectionManager.customDns()?.disable()
                    }
                }
            }
        }
    }

    private fun updateUi(settings: Settings) {
        if (this::wireguardMtuInput.isInitialized && wireguardMtuInput.hasFocus == false) {
            wireguardMtuInput.value = settings.tunnelOptions.wireguard.mtu
        }
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
}
