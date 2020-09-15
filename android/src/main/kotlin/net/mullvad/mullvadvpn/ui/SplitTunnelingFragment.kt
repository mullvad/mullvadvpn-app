package net.mullvad.mullvadvpn.ui

import android.animation.Animator
import android.animation.Animator.AnimatorListener
import android.animation.ObjectAnimator
import android.os.Bundle
import android.support.v7.widget.LinearLayoutManager
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.applist.AppListAdapter
import net.mullvad.mullvadvpn.ui.widget.CellSwitch
import net.mullvad.mullvadvpn.ui.widget.CustomRecyclerView
import net.mullvad.mullvadvpn.ui.widget.ToggleCell
import net.mullvad.mullvadvpn.util.AdapterWithHeader

class SplitTunnelingFragment : ServiceDependentFragment(OnNoService.GoToLaunchScreen) {
    private val excludeApplicationsFadeOutListener = object : AnimatorListener {
        override fun onAnimationCancel(animation: Animator) {}
        override fun onAnimationRepeat(animation: Animator) {}
        override fun onAnimationStart(animation: Animator) {}

        override fun onAnimationEnd(animation: Animator) {
            if (!appListAdapter.enabled && appListAdapter.isListReady) {
                excludeApplications.visibility = View.GONE
            }
        }
    }

    private val loadingSpinnerFadeOutListener = object : AnimatorListener {
        override fun onAnimationCancel(animation: Animator) {}
        override fun onAnimationRepeat(animation: Animator) {}
        override fun onAnimationStart(animation: Animator) {}

        override fun onAnimationEnd(animation: Animator) {
            if (appListAdapter.isListReady) {
                appListAdapter.enabled = true
                loadingSpinner.visibility = View.GONE
            }
        }
    }

    private lateinit var appListAdapter: AppListAdapter
    private lateinit var enabledToggle: ToggleCell
    private lateinit var excludeApplicationsFadeOut: ObjectAnimator
    private lateinit var loadingSpinnerFadeIn: ObjectAnimator
    private lateinit var titleController: CollapsibleTitleController

    private lateinit var excludeApplications: View
    private lateinit var loadingSpinner: View

    override fun onSafelyCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.split_tunneling, container, false)

        view.findViewById<View>(R.id.back).setOnClickListener {
            activity?.onBackPressed()
        }

        titleController = CollapsibleTitleController(view, R.id.app_list)

        appListAdapter = AppListAdapter(parentActivity, splitTunneling)

        view.findViewById<CustomRecyclerView>(R.id.app_list).apply {
            layoutManager = LinearLayoutManager(parentActivity)

            adapter = AdapterWithHeader(appListAdapter, R.layout.split_tunneling_header).apply {
                onHeaderAvailable = { headerView ->
                    configureHeader(headerView)
                    titleController.expandedTitleView = headerView.findViewById(R.id.expanded_title)
                }
            }

            addItemDecoration(
                ListItemDividerDecoration(parentActivity).apply {
                    bottomOffsetId = R.dimen.list_item_divider
                }
            )
        }

        return view
    }

    override fun onSafelyStop() {
        jobTracker.newBackgroundJob("persistExcludedApps") {
            splitTunneling.persist()
        }
    }

    override fun onSafelyDestroyView() {
        titleController.onDestroy()
    }

    private fun configureHeader(header: View) {
        excludeApplications = header.findViewById(R.id.exclude_applications)
        loadingSpinner = header.findViewById(R.id.loading_spinner)

        excludeApplicationsFadeOut =
            ObjectAnimator.ofFloat(excludeApplications, "alpha", 1.0f, 0.0f).apply {
                addListener(excludeApplicationsFadeOutListener)
                setDuration(200)
            }

        loadingSpinnerFadeIn =
            ObjectAnimator.ofFloat(loadingSpinner, "alpha", 0.0f, 1.0f).apply {
                addListener(loadingSpinnerFadeOutListener)
                setDuration(200)
            }

        if (configureSpinner()) {
            jobTracker.newUiJob("enableAdapter") {
                loadingSpinner.visibility = View.GONE
                appListAdapter.enabled = true
            }
        }

        if (splitTunneling.enabled) {
            jobTracker.newUiJob("showExcludedApplications") {
                excludeApplications.visibility = View.VISIBLE
            }
        }

        enabledToggle = header.findViewById<ToggleCell>(R.id.enabled).apply {
            if (splitTunneling.enabled) {
                forcefullySetState(CellSwitch.State.ON)
            } else {
                forcefullySetState(CellSwitch.State.OFF)
            }

            listener = { toggleState ->
                when (toggleState) {
                    CellSwitch.State.ON -> enable()
                    CellSwitch.State.OFF -> disable()
                }
            }
        }

        header.findViewById<View>(R.id.enabled).setOnClickListener {
            enabledToggle.toggle()
        }
    }

    private fun enable() {
        splitTunneling.enabled = true
        appListAdapter.enabled = configureSpinner()
        excludeApplications.visibility = View.VISIBLE
        excludeApplicationsFadeOut.reverse()
    }

    private fun disable() {
        splitTunneling.enabled = false
        appListAdapter.enabled = false
        excludeApplicationsFadeOut.start()
    }

    private fun configureSpinner(): Boolean {
        if (splitTunneling.enabled && !appListAdapter.isListReady) {
            showLoadingSpinner()

            appListAdapter.onListReady = {
                hideLoadingSpinner()
            }

            return false
        } else {
            return splitTunneling.enabled
        }
    }

    private fun showLoadingSpinner() {
        loadingSpinner.visibility = View.VISIBLE
        loadingSpinnerFadeIn.start()
    }

    private fun hideLoadingSpinner() {
        loadingSpinnerFadeIn.reverse()
    }
}
