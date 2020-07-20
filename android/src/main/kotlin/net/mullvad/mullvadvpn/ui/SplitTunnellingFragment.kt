package net.mullvad.mullvadvpn.ui

import android.animation.Animator
import android.animation.Animator.AnimatorListener
import android.animation.ObjectAnimator
import android.content.Context
import android.os.Bundle
import android.support.v7.widget.LinearLayoutManager
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.applist.AppListAdapter
import net.mullvad.mullvadvpn.ui.widget.CustomRecyclerView
import net.mullvad.mullvadvpn.util.AdapterWithHeader

class SplitTunnellingFragment : ServiceDependentFragment(OnNoService.GoToLaunchScreen) {
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
    private lateinit var enabledToggle: CellSwitch
    private lateinit var excludeApplicationsFadeOut: ObjectAnimator
    private lateinit var loadingSpinnerFadeIn: ObjectAnimator
    private lateinit var titleController: CollapsibleTitleController

    private lateinit var excludeApplications: View
    private lateinit var loadingSpinner: View

    override fun onAttach(context: Context) {
        super.onAttach(context)

        appListAdapter = AppListAdapter(context, splitTunnelling)
    }

    override fun onSafelyCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.split_tunnelling, container, false)

        view.findViewById<View>(R.id.back).setOnClickListener {
            activity?.onBackPressed()
        }

        titleController = CollapsibleTitleController(view, R.id.app_list)

        view.findViewById<CustomRecyclerView>(R.id.app_list).apply {
            layoutManager = LinearLayoutManager(parentActivity)

            adapter = AdapterWithHeader(appListAdapter, R.layout.split_tunnelling_header).apply {
                onHeaderAvailable = { headerView ->
                    configureHeader(headerView)
                    titleController.expandedTitleView = headerView.findViewById(R.id.expanded_title)
                }
            }

            addItemDecoration(ListItemDividerDecoration(parentActivity))
        }

        return view
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

        enabledToggle = header.findViewById<CellSwitch>(R.id.enabled_toggle).apply {
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
        appListAdapter.apply {
            if (!isListReady) {
                enabled = false
                showLoadingSpinner()
                onListReady = {
                    hideLoadingSpinner()
                }
            } else {
                enabled = true
            }
        }

        excludeApplications.visibility = View.VISIBLE
        excludeApplicationsFadeOut.reverse()
        splitTunnelling.enabled = true
    }

    private fun disable() {
        appListAdapter.enabled = false
        splitTunnelling.enabled = false
        excludeApplicationsFadeOut.start()
    }

    private fun showLoadingSpinner() {
        loadingSpinner.visibility = View.VISIBLE
        loadingSpinnerFadeIn.start()
    }

    private fun hideLoadingSpinner() {
        loadingSpinnerFadeIn.reverse()
    }
}
