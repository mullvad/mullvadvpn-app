package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.recyclerview.widget.LinearLayoutManager
import java.net.InetAddress
import kotlinx.coroutines.CompletableDeferred
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.ui.customdns.CustomDnsAdapter
import net.mullvad.mullvadvpn.ui.fragments.SplitTunnelingFragment
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnection
import net.mullvad.mullvadvpn.ui.widget.CellSwitch
import net.mullvad.mullvadvpn.ui.widget.CustomRecyclerView
import net.mullvad.mullvadvpn.ui.widget.MtuCell
import net.mullvad.mullvadvpn.ui.widget.NavigateCell
import net.mullvad.mullvadvpn.ui.widget.ToggleCell
import net.mullvad.mullvadvpn.util.AdapterWithHeader

class AdvancedFragment : ServiceDependentFragment(OnNoService.GoBack) {
    private var isAllowLanEnabled = false

    // Both customDnsAdapter and customDnsToggle are nullable since onNewServiceConnection,
    // which sets up custom dns subscriptions, is called before onSafelyCreateView.
    private var customDnsAdapter: CustomDnsAdapter? = null
    private var customDnsToggle: ToggleCell? = null

    private lateinit var wireguardMtuInput: MtuCell
    private lateinit var titleController: CollapsibleTitleController

    override fun onSafelyCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.advanced, container, false)

        view.findViewById<View>(R.id.back).setOnClickListener {
            customDnsAdapter?.stopEditing()
            parentActivity.onBackPressed()
        }

        titleController = CollapsibleTitleController(view, R.id.contents)

        customDnsAdapter = CustomDnsAdapter(
            onAddServer = { address -> customDns.addDnsServer(address) },
            onRemoveDnsServer = { address -> customDns.removeDnsServer(address) },
            onSetCustomDnsEnabled = { isEnabled ->
                if (isEnabled) {
                    customDns.enable()
                } else {
                    customDns.disable()
                }
            },
            onReplaceDnsServer = { oldServer, newServer ->
                customDns.replaceDnsServer(oldServer, newServer)
            }
        ).also { newCustomDnsAdapter ->
            newCustomDnsAdapter.confirmAddAddress = ::confirmAddAddress

            view.findViewById<CustomRecyclerView>(R.id.contents).apply {
                layoutManager = LinearLayoutManager(parentActivity)

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

    override fun onNewServiceConnection(serviceConnection: ServiceConnection) {
        super.onNewServiceConnection(serviceConnection)
        subscribeToCustomDnsChanges()
    }

    override fun onSafelyDestroyView() {
        detachBackButtonHandler()
        customDnsAdapter?.onDestroy()
        titleController.onDestroy()
        settingsListener.settingsNotifier.unsubscribe(this)
    }

    private fun configureHeader(view: View) {
        wireguardMtuInput = view.findViewById<MtuCell>(R.id.wireguard_mtu).apply {
            onSubmit = { mtu -> settingsListener.wireguardMtu = mtu }
        }

        view.findViewById<NavigateCell>(R.id.split_tunneling).apply {
            targetFragment = SplitTunnelingFragment::class
        }

        customDnsToggle = view.findViewById<ToggleCell>(R.id.enable_custom_dns).apply {
            listener = { state ->
                jobTracker.newBackgroundJob("toggleCustomDns") {
                    if (state == CellSwitch.State.ON) {
                        customDns.enable()
                    } else {
                        customDns.disable()
                    }
                }
            }
        }

        settingsListener.settingsNotifier.subscribe(this) { maybeSettings ->
            maybeSettings?.let { settings ->
                updateUi(settings)
            }

            isAllowLanEnabled = maybeSettings?.allowLan ?: false
        }

        subscribeToCustomDnsChanges()
    }

    private fun subscribeToCustomDnsChanges() {
        // Ensure there are no previous subscriptions as this function might be called either when
        // there view has been created or when there is a new service connection.
        customDns.onEnabledChanged.unsubscribe(this)
        customDns.onDnsServersChanged.unsubscribe(this)

        customDns.onEnabledChanged.subscribe(this) { isEnabled ->
            customDnsAdapter?.updateState(isEnabled)
            jobTracker.newUiJob("updateEnabled") {
                if (isEnabled) {
                    customDnsToggle?.state = CellSwitch.State.ON
                } else {
                    customDnsToggle?.state = CellSwitch.State.OFF
                }
            }
        }

        customDns.onDnsServersChanged.subscribe(this) { servers ->
            customDnsAdapter?.updateServers(servers)
        }
    }

    private fun updateUi(settings: Settings) {
        jobTracker.newUiJob("updateUi") {
            if (!wireguardMtuInput.hasFocus) {
                wireguardMtuInput.value = settings.tunnelOptions.wireguard.options.mtu
            }
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
        parentActivity.backButtonHandler = {
            if (customDnsAdapter?.isEditing == true) {
                customDnsAdapter?.stopEditing()
            }
            false
        }
    }

    private fun detachBackButtonHandler() {
        parentActivity.backButtonHandler = null
    }
}
