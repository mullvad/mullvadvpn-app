package net.mullvad.mullvadvpn

import kotlinx.coroutines.launch
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import android.content.Context
import android.os.Bundle
import android.support.v4.app.Fragment
import android.support.v7.widget.LinearLayoutManager
import android.support.v7.widget.RecyclerView
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.ImageButton
import android.widget.ViewSwitcher

import net.mullvad.mullvadvpn.dataproxy.ConnectionProxy
import net.mullvad.mullvadvpn.dataproxy.RelayListListener
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.LocationConstraint
import net.mullvad.mullvadvpn.model.RelaySettingsUpdate
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.RelayItemDividerDecoration
import net.mullvad.mullvadvpn.relaylist.RelayList
import net.mullvad.mullvadvpn.relaylist.RelayListAdapter
import net.mullvad.mullvadvpn.util.SmartDeferred

class SelectLocationFragment : Fragment() {
    private lateinit var parentActivity: MainActivity
    private lateinit var connectionProxy: SmartDeferred<ConnectionProxy>
    private lateinit var relayListListener: RelayListListener

    private lateinit var relayListContainer: ViewSwitcher

    private lateinit var relayListAdapter: RelayListAdapter

    private var updateRelayListJob: Job? = null

    override fun onAttach(context: Context) {
        super.onAttach(context)

        parentActivity = context as MainActivity
        connectionProxy = parentActivity.connectionProxy
        relayListListener = parentActivity.relayListListener

        relayListAdapter = RelayListAdapter(context.resources).apply {
            onSelect = { relayItem ->
                updateLocationConstraint(relayItem)
                maybeConnect()
                close()
            }
        }
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.select_location, container, false)

        view.findViewById<ImageButton>(R.id.close).setOnClickListener { close() }

        relayListContainer = view.findViewById<ViewSwitcher>(R.id.relay_list_container)
        relayListContainer.showNext()

        configureRelayList(view.findViewById<RecyclerView>(R.id.relay_list))

        return view
    }

    override fun onResume() {
        super.onResume()

        relayListListener.onRelayListChange = { relayList, selectedItem ->
            updateRelayListJob = updateRelayList(relayList, selectedItem)
        }
    }

    override fun onPause() {
        relayListListener.onRelayListChange = null

        super.onPause()
    }

    override fun onDestroyView() {
        updateRelayListJob?.cancel()

        super.onDestroyView()
    }

    fun close() {
        activity?.onBackPressed()
    }

    private fun configureRelayList(relayList: RecyclerView) {
        relayList.apply {
            layoutManager = LinearLayoutManager(context!!)
            adapter = relayListAdapter

            addItemDecoration(RelayItemDividerDecoration(context!!))
        }
    }

    private fun updateLocationConstraint(relayItem: RelayItem?) =
            GlobalScope.launch(Dispatchers.Default) {
        val constraint: Constraint<LocationConstraint> =
            relayItem?.run { Constraint.Only(location) } ?: Constraint.Any()

        parentActivity.daemon.await().updateRelaySettings(
            RelaySettingsUpdate.RelayConstraintsUpdate(constraint)
        )
    }

    private fun updateRelayList(relayList: RelayList, selectedItem: RelayItem?) =
            GlobalScope.launch(Dispatchers.Main) {
        relayListAdapter.onRelayListChange(relayList, selectedItem)

        if (relayList.countries.isEmpty()) {
            relayListContainer.showPrevious()
        } else if (relayListContainer.displayedChild == 0) {
            relayListContainer.showNext()
        }
    }

    private fun maybeConnect() {
        val keyStatus = parentActivity.keyStatusListener.keyStatus

        if (keyStatus == null || keyStatus is KeygenEvent.NewKey) {
            connectionProxy.awaitThen { connect() }
        }
    }
}
