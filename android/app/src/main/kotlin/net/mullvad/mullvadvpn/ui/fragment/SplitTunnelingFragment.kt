package net.mullvad.mullvadvpn.ui.fragment

import android.os.Build
import android.os.Bundle
import android.view.KeyCharacterMap
import android.view.KeyEvent
import android.view.View
import android.view.ViewConfiguration
import androidx.lifecycle.lifecycleScope
import androidx.recyclerview.widget.RecyclerView
import com.google.android.material.appbar.CollapsingToolbarLayout
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.consumeAsFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.onEach
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.applist.ViewIntent
import net.mullvad.mullvadvpn.di.APPS_SCOPE
import net.mullvad.mullvadvpn.di.SERVICE_CONNECTION_SCOPE
import net.mullvad.mullvadvpn.model.ListItemData
import net.mullvad.mullvadvpn.model.WidgetState.ImageState
import net.mullvad.mullvadvpn.model.WidgetState.SwitchState
import net.mullvad.mullvadvpn.ui.ListItemDividerDecoration
import net.mullvad.mullvadvpn.ui.ListItemListener
import net.mullvad.mullvadvpn.ui.ListItemsAdapter
import net.mullvad.mullvadvpn.util.setMargins
import net.mullvad.mullvadvpn.viewmodel.SplitTunnelingViewModel
import org.koin.android.ext.android.getKoin
import org.koin.androidx.viewmodel.ViewModelOwner
import org.koin.androidx.viewmodel.scope.viewModel
import org.koin.core.qualifier.named
import org.koin.core.scope.Scope

class SplitTunnelingFragment : BaseFragment(R.layout.collapsed_title_layout) {
    private val listItemsAdapter = ListItemsAdapter()
    private val scope: Scope = getKoin().getOrCreateScope(APPS_SCOPE, named(APPS_SCOPE))
        .also { appsScope ->
            getKoin().getScopeOrNull(SERVICE_CONNECTION_SCOPE)?.let { serviceConnectionScope ->
                appsScope.linkTo(serviceConnectionScope)
            }
        }
    private val viewModel by scope.viewModel<SplitTunnelingViewModel>(
        owner = {
            ViewModelOwner.from(this, this)
        }
    )
    private val toggleSystemAppsVisibility = Channel<Boolean>(Channel.CONFLATED)
    private val toggleExcludeChannel = Channel<ListItemData>(Channel.BUFFERED)
    private val listItemListener = object : ListItemListener {
        override fun onItemAction(item: ListItemData) {
            when (item.widget) {
                is ImageState -> toggleExcludeChannel.trySend(item)
                is SwitchState -> toggleSystemAppsVisibility.trySend(!item.widget.isChecked)
                else -> { /* NOOP */ }
            }
        }
    }

    private var recyclerView: RecyclerView? = null

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)
        view.findViewById<CollapsingToolbarLayout>(R.id.collapsing_toolbar).apply {
            title = resources.getString(R.string.split_tunneling)
        }
        listItemsAdapter.listItemListener = listItemListener
        listItemsAdapter.setHasStableIds(true)
        recyclerView = view.findViewById<RecyclerView>(R.id.recyclerView).apply {
            adapter = listItemsAdapter
            addItemDecoration(
                ListItemDividerDecoration(
                    topOffset = resources.getDimensionPixelSize(R.dimen.list_item_divider)
                )
            )
            tweakMargin(this)
        }
        view.findViewById<View>(R.id.back).setOnClickListener {
            requireActivity().onBackPressed()
        }

        lifecycleScope.launchWhenStarted {
            viewModel.listItems
                .onEach {
                    listItemsAdapter.setItems(it)
                }
                .catch { }
                .collect()
        }
        lifecycleScope.launchWhenResumed {
            // pass view intent to view model
            intents()
                .onEach { viewModel.processIntent(it) }
                .collect()
        }
    }

    override fun onDestroy() {
        listItemsAdapter.listItemListener = null
        recyclerView?.adapter = null
        scope.close()
        super.onDestroy()
    }

    private fun intents(): Flow<ViewIntent> = merge(
        transitionFinishedFlow.map { ViewIntent.ViewIsReady },
        toggleExcludeChannel.consumeAsFlow().map { ViewIntent.ChangeApplicationGroup(it) },
        toggleSystemAppsVisibility.consumeAsFlow().map { ViewIntent.ShowSystemApps(it) }
    )

    private fun tweakMargin(view: View) {
        if (!hasNavigationBar()) {
            view.setMargins(b = 0)
        }
    }

    private fun hasNavigationBar(): Boolean {
        // Emulator
        if (Build.FINGERPRINT.contains("generic")) {
            return true
        }

        val hasMenuKey = ViewConfiguration.get(requireContext()).hasPermanentMenuKey()
        val hasBackKey = KeyCharacterMap.deviceHasKey(KeyEvent.KEYCODE_BACK)
        val hasNoCapacitiveKeys = !hasMenuKey && !hasBackKey

        val id = resources.getIdentifier("config_showNavigationBar", "bool", "android")
        val hasOnScreenNavBar = id > 0 && resources.getBoolean(id)

        return hasOnScreenNavBar || hasNoCapacitiveKeys
    }
}
