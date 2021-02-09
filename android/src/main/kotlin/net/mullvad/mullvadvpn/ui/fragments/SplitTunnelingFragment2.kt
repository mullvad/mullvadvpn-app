package net.mullvad.mullvadvpn.ui.fragments

import android.os.Build
import android.os.Bundle
import android.util.Log
import android.view.KeyCharacterMap
import android.view.KeyEvent
import android.view.View
import android.view.ViewConfiguration
import android.view.animation.Animation
import android.view.animation.AnimationUtils
import androidx.core.view.ViewCompat
import androidx.fragment.app.Fragment
import androidx.lifecycle.lifecycleScope
import androidx.recyclerview.widget.RecyclerView
import com.google.android.material.appbar.CollapsingToolbarLayout
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.*
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.applist.ProgressListItemAnimator
import net.mullvad.mullvadvpn.applist.ViewIntent
import net.mullvad.mullvadvpn.model.ListItemData
import net.mullvad.mullvadvpn.ui.ListItemDividerDecoration
import net.mullvad.mullvadvpn.ui.ListItemListener
import net.mullvad.mullvadvpn.ui.ListItemsAdapter
import net.mullvad.mullvadvpn.util.setMargins
import net.mullvad.mullvadvpn.viewmodel.SplitTunnelingViewModel
import org.koin.androidx.viewmodel.ext.android.viewModel

class SplitTunnelingFragment2 : Fragment(R.layout.collapsed_title_layout) {

    private val listItemsAdapter = ListItemsAdapter()

    private val viewModel by viewModel<SplitTunnelingViewModel>()
    private val toggleExcludeChannel = Channel<ListItemData>(Channel.BUFFERED)
    private val listItemListener = object : ListItemListener {
        override fun onItemAction(item: ListItemData) {
            toggleExcludeChannel.offer(item)
        }
    }

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)
        Log.e(this.javaClass.simpleName, "onViewCreated | ${this.hashCode()}")
        val collapsingToolbar = view.findViewById(R.id.collapsing_toolbar) as CollapsingToolbarLayout
        collapsingToolbar.title = resources.getString(R.string.split_tunneling)
        listItemsAdapter.listItemListener = listItemListener
        view.findViewById<RecyclerView>(R.id.recyclerView).apply {
            adapter = listItemsAdapter
            addItemDecoration(
                ListItemDividerDecoration(requireContext()).apply {
                    topOffsetId = R.dimen.list_item_divider
                }
            )
            tweakMargin(this)
            itemAnimator = ProgressListItemAnimator()
        }
        view.findViewById<View>(R.id.back).setOnClickListener {
            requireActivity().onBackPressed()
        }

        lifecycleScope.launchWhenStarted {
            viewModel.data
                .onEach {
                    listItemsAdapter.setItems(it)
                }
                .catch { }
                .collect()
        }

        // pass view intent to view model
        intents()
            .onEach { viewModel.processIntent(it) }
            .launchIn(lifecycleScope)
    }

    private fun intents(): Flow<ViewIntent> = merge(
        toggleExcludeChannel.consumeAsFlow().map { ViewIntent.ChangeApplicationGroup(it) }
    )

    private fun tweakMargin(view: View) {
        if (!hasNavigationBar()) {
            Log.e("test", "set padding 0 for RecyclerView")
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

        Log.e("test", "hasOnScreenNavBar=$hasOnScreenNavBar hasNoCapacitiveKeys=$hasNoCapacitiveKeys")
        return hasOnScreenNavBar || hasNoCapacitiveKeys
    }

    override fun onCreateAnimation(transit: Int, enter: Boolean, nextAnim: Int): Animation? {
        ViewCompat.setTranslationZ(requireView(), 1f)
        if (nextAnim != 0 && enter) {
            val animation = AnimationUtils.loadAnimation(context, nextAnim)
            Log.e("test", "animation = $animation")
            Log.e("test", "setListener")
            animation.setAnimationListener(object : Animation.AnimationListener {
                override fun onAnimationRepeat(animation: Animation?) {}

                override fun onAnimationEnd(animation: Animation?) {
                    Log.e("test", "animation end")
                    // viewModel.fetchData()
                }

                override fun onAnimationStart(animation: Animation?) {
                    Log.e("test", "animation start")
                }
            })
            return animation
        }
        return super.onCreateAnimation(transit, enter, nextAnim)
    }
}
