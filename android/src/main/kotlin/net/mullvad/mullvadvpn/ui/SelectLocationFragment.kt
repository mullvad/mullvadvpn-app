package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.os.Bundle
import android.support.v7.widget.LinearLayoutManager
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.view.animation.Animation
import android.view.animation.Animation.AnimationListener
import android.view.animation.AnimationUtils
import android.widget.ImageButton
import kotlinx.coroutines.CompletableDeferred
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.LocationConstraint
import net.mullvad.mullvadvpn.model.RelayConstraintsUpdate
import net.mullvad.mullvadvpn.model.RelaySettingsUpdate
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.RelayList
import net.mullvad.mullvadvpn.relaylist.RelayListAdapter
import net.mullvad.mullvadvpn.ui.widget.CustomRecyclerView
import net.mullvad.mullvadvpn.util.AdapterWithHeader

class SelectLocationFragment : ServiceDependentFragment(OnNoService.GoToLaunchScreen) {
    private enum class RelayListState {
        Initializing,
        Loading,
        Visible,
    }

    private lateinit var relayListAdapter: RelayListAdapter
    private lateinit var titleController: CollapsibleTitleController

    private var loadingSpinner = CompletableDeferred<View>()
    private var relayListState = RelayListState.Initializing

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

        titleController = CollapsibleTitleController(view, R.id.relay_list)

        view.findViewById<CustomRecyclerView>(R.id.relay_list).apply {
            layoutManager = LinearLayoutManager(parentActivity)

            adapter = AdapterWithHeader(relayListAdapter, R.layout.select_location_header).apply {
                onHeaderAvailable = { headerView ->
                    initializeLoadingSpinner(headerView)
                    titleController.expandedTitleView = headerView.findViewById(R.id.expanded_title)
                }
            }

            addItemDecoration(ListItemDividerDecoration(parentActivity))
        }

        return view
    }

    override fun onSafelyStart() {
        // If the relay list is immediately available, setting the listener will cause it to be
        // called right away, while the state is still Initializing. In that case we can skip
        // showing the spinner animation and go directly to the Visible state.
        //
        // If it's not immediately available, then when the listener is called later the state will
        // have changed to Loading, and an animation from the spinner to the new relay items will be
        // shown.
        //
        // If the state is ready, it means that the relay list has already been shown, and we can
        // update it in place.
        relayListListener.onRelayListChange = { relayList, selectedItem ->
            when (relayListState) {
                RelayListState.Initializing -> {
                    jobTracker.newUiJob("updateRelayList") {
                        updateRelayList(relayList, selectedItem)
                    }

                    relayListState = RelayListState.Visible
                }
                RelayListState.Loading -> {
                    jobTracker.newUiJob("updateRelayList") {
                        animateRelayListInitialization(relayList, selectedItem)
                    }
                }
                RelayListState.Visible -> {
                    jobTracker.newUiJob("updateRelayList") {
                        updateRelayList(relayList, selectedItem)
                    }
                }
            }
        }

        if (relayListState == RelayListState.Initializing) {
            relayListState = RelayListState.Loading
        }
    }

    override fun onSafelyStop() {
        relayListListener.onRelayListChange = null
    }

    override fun onSafelyDestroyView() {
        titleController.onDestroy()
    }

    fun close() {
        activity?.onBackPressed()
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

    private fun initializeLoadingSpinner(parentView: View) {
        val spinner = parentView.findViewById<View>(R.id.loading_spinner)

        if (relayListState == RelayListState.Visible) {
            // Because this method is executed inside a layout pass, hiding the spinner needs to be
            // done in a new job so that it is executed after the layout pass finishes and can
            // therefore schedule a new layout
            jobTracker.newUiJob("hideLoadingSpinner") {
                spinner.visibility = View.GONE
            }
        }

        loadingSpinner.complete(spinner)
    }

    // Smoothly fade out the spinner before showing the relay list items.
    private suspend fun animateRelayListInitialization(
        relayList: RelayList,
        selectedItem: RelayItem?
    ) {
        val animationFinished = CompletableDeferred<Unit>()
        val animationListener = object : AnimationListener {
            override fun onAnimationEnd(animation: Animation) {
                animationFinished.complete(Unit)
            }

            override fun onAnimationStart(animation: Animation) {}
            override fun onAnimationRepeat(animation: Animation) {}
        }

        val fadeOut = AnimationUtils.loadAnimation(parentActivity, R.anim.fade_out).apply {
            setAnimationListener(animationListener)
        }

        loadingSpinner.await().let { spinner ->
            spinner.startAnimation(fadeOut)

            animationFinished.await()

            spinner.visibility = View.GONE
            updateRelayList(relayList, selectedItem)
        }
    }
}
