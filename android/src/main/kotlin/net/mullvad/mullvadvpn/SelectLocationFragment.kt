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

import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.LocationConstraint
import net.mullvad.mullvadvpn.model.RelaySettingsUpdate
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.RelayItemDividerDecoration
import net.mullvad.mullvadvpn.relaylist.RelayList
import net.mullvad.mullvadvpn.relaylist.RelayListAdapter
import net.mullvad.mullvadvpn.relaylist.RelayListListener

class SelectLocationFragment : Fragment() {
    private lateinit var parentActivity: MainActivity
    private lateinit var relayListListener: RelayListListener

    private lateinit var relayListContainer: ViewSwitcher

    private val relayListAdapter = RelayListAdapter()

    private var updateRelayListJob: Job? = null

    init {
        relayListAdapter.onSelect = { relayItem ->
            relayListListener.selectedRelayItem = relayItem
            updateLocationConstraint()
            close()
        }
    }

    override fun onAttach(context: Context) {
        super.onAttach(context)

        parentActivity = context as MainActivity
        relayListListener = parentActivity.relayListListener

        relayListListener.onRelayListChange = { relayList, selectedItem ->
            updateRelayListJob = updateRelayList(relayList, selectedItem)
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

    private fun updateLocationConstraint() = GlobalScope.launch(Dispatchers.Default) {
        val constraint = relayListListener.selectedRelayLocation

        parentActivity.asyncDaemon.await().updateRelaySettings(
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
}
