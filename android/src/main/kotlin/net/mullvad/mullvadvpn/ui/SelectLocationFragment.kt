package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.os.Bundle
import android.support.v7.widget.LinearLayoutManager
import android.support.v7.widget.RecyclerView
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.ImageButton
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.LocationConstraint
import net.mullvad.mullvadvpn.model.RelayConstraintsUpdate
import net.mullvad.mullvadvpn.model.RelaySettingsUpdate
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.RelayItemDividerDecoration
import net.mullvad.mullvadvpn.relaylist.RelayList
import net.mullvad.mullvadvpn.relaylist.RelayListAdapter
import net.mullvad.mullvadvpn.util.AdapterWithHeader

class SelectLocationFragment : ServiceDependentFragment(OnNoService.GoToLaunchScreen) {
    private lateinit var relayListAdapter: RelayListAdapter

    override fun onAttach(context: Context) {
        super.onAttach(context)

        relayListAdapter = RelayListAdapter(context.resources).apply {
            onSelect = { relayItem ->
                jobTracker.newBackgroundJob("selectRelay") {
                    updateLocationConstraint(relayItem)
                    maybeConnect()

                    jobTracker.newUiJob("close") {
                        close()
                    }
                }
            }
        }
    }

    override fun onSafelyCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.select_location, container, false)

        view.findViewById<ImageButton>(R.id.close).setOnClickListener { close() }

        view.findViewById<RecyclerView>(R.id.relay_list).apply {
            layoutManager = LinearLayoutManager(parentActivity)
            adapter = AdapterWithHeader(relayListAdapter, R.layout.select_location_header)

            addItemDecoration(RelayItemDividerDecoration(parentActivity))
        }

        return view
    }

    override fun onSafelyResume() {
        relayListListener.onRelayListChange = { relayList, selectedItem ->
            jobTracker.newUiJob("updateRelayList") {
                updateRelayList(relayList, selectedItem)
            }
        }
    }

    override fun onSafelyPause() {
        relayListListener.onRelayListChange = null
    }

    fun close() {
        activity?.onBackPressed()
    }

    }

    private fun updateLocationConstraint(relayItem: RelayItem?) {
        val constraint: Constraint<LocationConstraint> =
            relayItem?.run { Constraint.Only(location) } ?: Constraint.Any()

        daemon.updateRelaySettings(RelaySettingsUpdate.Normal(RelayConstraintsUpdate(constraint)))
    }

    private fun updateRelayList(relayList: RelayList, selectedItem: RelayItem?) {
        relayListAdapter.onRelayListChange(relayList, selectedItem)
    }

    private fun maybeConnect() {
        val keyStatus = keyStatusListener.keyStatus

        if (keyStatus == null || keyStatus is KeygenEvent.NewKey) {
            connectionProxy.connect()
        }
    }
}
