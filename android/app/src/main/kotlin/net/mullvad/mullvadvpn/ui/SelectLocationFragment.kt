package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.view.animation.Animation
import android.view.animation.Animation.AnimationListener
import android.view.animation.AnimationUtils
import android.widget.ImageButton
import androidx.core.content.ContextCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import androidx.recyclerview.widget.LinearLayoutManager
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.RelayList
import net.mullvad.mullvadvpn.relaylist.RelayListAdapter
import net.mullvad.mullvadvpn.ui.extension.requireMainActivity
import net.mullvad.mullvadvpn.ui.fragment.BaseFragment
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.connectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.relayListListener
import net.mullvad.mullvadvpn.ui.widget.CustomRecyclerView
import net.mullvad.mullvadvpn.util.AdapterWithHeader
import net.mullvad.mullvadvpn.util.JobTracker
import org.koin.android.ext.android.inject

class SelectLocationFragment : BaseFragment(), StatusBarPainter, NavigationBarPainter {

    // Injected dependencies
    private val serviceConnectionManager: ServiceConnectionManager by inject()

    private enum class RelayListState {
        Initializing,
        Loading,
        Visible,
    }

    private lateinit var relayListAdapter: RelayListAdapter
    private lateinit var titleController: CollapsibleTitleController

    private var loadingSpinner = CompletableDeferred<View>()
    private var relayListState = RelayListState.Initializing

    @Deprecated("Refactor code to instead rely on Lifecycle.")
    private val jobTracker = JobTracker()

    override fun onAttach(context: Context) {
        super.onAttach(context)

        relayListAdapter = RelayListAdapter(context.resources).apply {
            onSelect = { relayItem ->
                serviceConnectionManager.relayListListener()?.selectedRelayLocation =
                    relayItem?.location
                serviceConnectionManager.connectionProxy()?.connect()
                close()
            }
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        lifecycleScope.launchUiSubscriptionsOnResume()
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        val view = inflater.inflate(R.layout.select_location, container, false)

        view.findViewById<ImageButton>(R.id.close).setOnClickListener { close() }

        titleController = CollapsibleTitleController(view, R.id.relay_list)

        view.findViewById<CustomRecyclerView>(R.id.relay_list).apply {
            layoutManager = LinearLayoutManager(requireMainActivity())

            adapter = AdapterWithHeader(relayListAdapter, R.layout.select_location_header).apply {
                onHeaderAvailable = { headerView ->
                    initializeLoadingSpinner(headerView)
                    titleController.expandedTitleView = headerView.findViewById(R.id.expanded_title)
                }
            }

            addItemDecoration(
                ListItemDividerDecoration(
                    bottomOffset = resources.getDimensionPixelSize(R.dimen.list_item_divider)
                )
            )
        }

        return view
    }

    override fun onResume() {
        super.onResume()
        paintNavigationBar(ContextCompat.getColor(requireContext(), R.color.darkBlue))
    }

    override fun onDestroyView() {
        titleController.onDestroy()
        super.onDestroyView()
    }

    fun close() {
        activity?.onBackPressed()
    }

    private fun CoroutineScope.launchUiSubscriptionsOnResume() = launch {
        repeatOnLifecycle(Lifecycle.State.RESUMED) {
            launchPaintStatusBarAfterTransition()
            launchRelayListSubscription()
        }
    }

    private fun CoroutineScope.launchPaintStatusBarAfterTransition() = launch {
        transitionFinishedFlow.collect {
            paintStatusBar(ContextCompat.getColor(requireContext(), R.color.darkBlue))
        }
    }

    private fun CoroutineScope.launchRelayListSubscription() = launch {
        serviceConnectionManager.connectionState
            .flatMapLatest { state ->
                if (state is ServiceConnectionState.ConnectedReady) {
                    callbackFlow {
                        state.container.relayListListener.onRelayListChange =
                            { list, item ->
                                this.trySend(Pair(list, item))
                            }

                        awaitClose {
                            state.container.relayListListener.onRelayListChange = null
                        }
                    }
                } else {
                    emptyFlow()
                }
            }
            .collect { (relayList, selectedItem) ->
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

                if (relayListState == RelayListState.Initializing) {
                    relayListState = RelayListState.Loading
                }
            }
    }

    private fun updateRelayList(relayList: RelayList, selectedItem: RelayItem?) {
        relayListAdapter.onRelayListChange(relayList, selectedItem)
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

        val fadeOut =
            AnimationUtils.loadAnimation(requireMainActivity(), R.anim.fade_out).apply {
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
