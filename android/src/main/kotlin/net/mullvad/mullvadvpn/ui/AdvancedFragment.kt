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
import net.mullvad.mullvadvpn.ui.widget.CellSwitch
import net.mullvad.mullvadvpn.ui.widget.CustomRecyclerView
import net.mullvad.mullvadvpn.ui.widget.MtuCell
import net.mullvad.mullvadvpn.ui.widget.NavigateCell
import net.mullvad.mullvadvpn.ui.widget.ToggleCell
import net.mullvad.mullvadvpn.util.AdapterWithHeader

class AdvancedFragment : ServiceDependentFragment(OnNoService.GoBack) {
    private lateinit var customDnsAdapter: CustomDnsAdapter
    private lateinit var customDnsToggle: ToggleCell
    private lateinit var wireguardMtuInput: MtuCell
    private lateinit var titleController: CollapsibleTitleController

    override fun onSafelyCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.advanced, container, false)

        view.findViewById<View>(R.id.back).setOnClickListener {
            customDnsAdapter.stopEditing()
            parentActivity.onBackPressed()
        }

        titleController = CollapsibleTitleController(view, R.id.contents)

        customDnsAdapter = CustomDnsAdapter(customDns).apply {
            confirmAddAddress = ::confirmAddAddress
        }

        view.findViewById<CustomRecyclerView>(R.id.contents).apply {
            layoutManager = LinearLayoutManager(parentActivity)

            adapter = AdapterWithHeader(customDnsAdapter, R.layout.advanced_header).apply {
                onHeaderAvailable = { headerView ->
                    configureHeader(headerView)
                    titleController.expandedTitleView = headerView.findViewById(R.id.expanded_title)
                }
            }

            addItemDecoration(
                ListItemDividerDecoration(parentActivity).apply {
                    topOffsetId = R.dimen.list_item_divider
                }
            )
        }

        attachBackButtonHandler()

        return view
    }

    override fun onSafelyDestroyView() {
        detachBackButtonHandler()
        customDnsAdapter.onDestroy()
        titleController.onDestroy()
        settingsListener.unsubscribe(this)
    }

    private fun configureHeader(view: View) {
        wireguardMtuInput = view.findViewById<MtuCell>(R.id.wireguard_mtu).apply {
            onSubmit = { mtu ->
                jobTracker.newBackgroundJob("updateMtu") {
                    daemon.setWireguardMtu(mtu)
                }
            }
        }

        view.findViewById<NavigateCell>(R.id.wireguard_keys).apply {
            targetFragment = WireguardKeyFragment::class
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

        customDns.onEnabledChanged.subscribe(this) { enabled ->
            jobTracker.newUiJob("updateEnabled") {
                if (enabled) {
                    customDnsToggle.state = CellSwitch.State.ON
                } else {
                    customDnsToggle.state = CellSwitch.State.OFF
                }
            }
        }

        settingsListener.subscribe(this) { settings ->
            updateUi(settings)
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
        if (address.isLinkLocalAddress() || address.isSiteLocalAddress()) {
            return true
        }

        val confirmation = CompletableDeferred<Boolean>()

        showConfirmDnsServerDialog(confirmation)

        return confirmation.await()
    }

    private fun showConfirmDnsServerDialog(confirmation: CompletableDeferred<Boolean>) {
        val transaction = parentFragmentManager.beginTransaction()

        detachBackButtonHandler()
        transaction.addToBackStack(null)

        ConfirmDnsDialogFragment(confirmation)
            .show(transaction, null)

        jobTracker.newUiJob("restoreBackButtonHandler") {
            confirmation.await()
            attachBackButtonHandler()
        }
    }

    private fun attachBackButtonHandler() {
        parentActivity.backButtonHandler = {
            if (customDnsAdapter.isEditing) {
                customDnsAdapter.stopEditing()
                true
            } else {
                false
            }
        }
    }

    private fun detachBackButtonHandler() {
        parentActivity.backButtonHandler = null
    }
}
