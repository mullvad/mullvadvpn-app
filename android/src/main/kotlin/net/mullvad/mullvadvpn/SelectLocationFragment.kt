package net.mullvad.mullvadvpn

import android.os.Bundle
import android.support.v4.app.Fragment
import android.support.v7.widget.LinearLayoutManager
import android.support.v7.widget.RecyclerView
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.ImageButton

import net.mullvad.mullvadvpn.relaylist.RelayItemDividerDecoration
import net.mullvad.mullvadvpn.relaylist.RelayList
import net.mullvad.mullvadvpn.relaylist.RelayListAdapter

class SelectLocationFragment : Fragment() {
    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.select_location, container, false)

        view.findViewById<ImageButton>(R.id.close).setOnClickListener { close() }

        configureRelayList(view.findViewById<RecyclerView>(R.id.relay_list))

        return view
    }

    fun close() {
        activity?.onBackPressed()
    }

    private fun configureRelayList(relayList: RecyclerView) {
        val parentActivity = activity as MainActivity
        val relayListAdapter =
            RelayListAdapter(parentActivity.relayList, parentActivity.selectedRelayItem)

        relayListAdapter.onSelect = { relayItem ->
            parentActivity.selectedRelayItem = relayItem
            close()
        }

        relayList.apply {
            layoutManager = LinearLayoutManager(context!!)
            adapter = relayListAdapter

            addItemDecoration(RelayItemDividerDecoration(context!!))
        }
    }
}
