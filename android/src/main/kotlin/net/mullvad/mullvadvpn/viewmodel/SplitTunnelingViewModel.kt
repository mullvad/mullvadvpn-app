package net.mullvad.mullvadvpn.viewmodel

import androidx.annotation.DrawableRes
import androidx.annotation.StringRes
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.applist.AppInfo
import net.mullvad.mullvadvpn.applist.ApplicationsProvider
import net.mullvad.mullvadvpn.applist.ViewIntent
import net.mullvad.mullvadvpn.model.ListItemData
import net.mullvad.mullvadvpn.model.WidgetState
import net.mullvad.mullvadvpn.service.SplitTunneling

class SplitTunnelingViewModel(
    private val appsProvider: ApplicationsProvider,
    private val splitTunneling: SplitTunneling
) : ViewModel() {
    private val listItemsSink = MutableSharedFlow<List<ListItemData>>(replay = 1)
    val listItems = listItemsSink.asSharedFlow() // read-only public view

    private val intentFlow = MutableSharedFlow<ViewIntent>()

    private lateinit var excludedApps: MutableMap<String, AppInfo>
    private lateinit var notExcludedApps: MutableMap<String, AppInfo>

    private val defaultListItems = mutableListOf(
        createTextItem(R.string.split_tunneling_description)
        // We will have search item in future
    )

    init {
        viewModelScope.launch(Dispatchers.Default) {
            listItemsSink.emit(defaultListItems + createDivider(0) + createProgressItem())
            // this will be removed after changes on native to ignore enable parameter
            if (!splitTunneling.enabled)
                splitTunneling.enabled = true
            fetchData()
            intentFlow.shareIn(viewModelScope, SharingStarted.WhileSubscribed())
                .collect { viewIntent ->
                    when (viewIntent) {
                        is ViewIntent.ChangeApplicationGroup -> {
                            viewIntent.item.action?.let {
                                if (excludedApps.containsKey(it.identifier)) {
                                    removeFromExcluded(it.identifier)
                                } else {
                                    addToExcluded(it.identifier)
                                }
                                publishList()
                            }
                        }
                    }
                }
        }
    }

    private fun removeFromExcluded(packageName: String) {
        excludedApps.apply {
            get(packageName)?.let { appInfo ->
                notExcludedApps[packageName] = appInfo
                splitTunneling.includeApp(packageName)
            }
            remove(packageName)
        }
    }

    private fun addToExcluded(packageName: String) {
        notExcludedApps.apply {
            get(packageName)?.let { appInfo ->
                excludedApps[packageName] = appInfo
                splitTunneling.excludeApp(packageName)
            }
            remove(packageName)
        }
    }

    private suspend fun fetchData() {

        appsProvider.getAppsList().map { it.packageName to it }.toMap().let { applications ->
            val excludedPackages = applications.keys.intersect(splitTunneling.excludedAppList)
            // TODO: remove potential package names from splitTunneling list
            //       if they already uninstalled or filtered
            excludedApps = applications
                .filterKeys { excludedPackages.contains(it) }
                .toMutableMap()
            notExcludedApps = applications
                .filterKeys { applications.keys.subtract(excludedPackages).contains(it) }
                .toMutableMap()
        }
        // Short delay for smooth transition animation, will not effect user experience
        delay(100)
        publishList()
    }

    private suspend fun publishList() {
        val listItems = mutableListOf<ListItemData>()
        listItems.addAll(defaultListItems)
        if (excludedApps.isNotEmpty()) {
            listItems.add(createDivider(0))
            listItems.add(createMainItem(R.string.exclude_applications))
            listItems.addAll(
                excludedApps.values.sortedBy { it.name }.map { info ->
                    createApplicationItem(info.packageName, info.name, info.iconRes, true)
                }
            )
        }

        if (notExcludedApps.isNotEmpty()) {
            listItems.add(createDivider(1))
            listItems.add(createMainItem(R.string.all_applications))
            listItems.addAll(
                notExcludedApps.values.sortedBy { it.name }.map { info ->
                    createApplicationItem(info.packageName, info.name, info.iconRes, false)
                }
            )
        }
        listItemsSink.emit(listItems)
    }

    private fun createApplicationItem(
        id: String,
        name: String,
        @DrawableRes icon: Int,
        checked: Boolean
    ): ListItemData = ListItemData.build {
        identifier = id
        type = ListItemData.APPLICATION
        text = name
        iconRes = icon
        action = ListItemData.ItemAction(id)
        widget = WidgetState.ImageState(
            if (checked) R.drawable.ic_icons_remove else R.drawable.ic_icons_add
        )
    }

    private fun createDivider(id: Int): ListItemData = ListItemData.build {
        identifier = "space_$id"
        type = ListItemData.DIVIDER
    }

    private fun createMainItem(@StringRes text: Int): ListItemData = ListItemData.build {
        identifier = "header_$text"
        type = ListItemData.ACTION
        textRes = text
    }

    private fun createTextItem(@StringRes text: Int): ListItemData = ListItemData.build {
        identifier = "text_$text"
        type = ListItemData.PLAIN
        textRes = text
        action = ListItemData.ItemAction(text.toString())
    }

    private fun createProgressItem(): ListItemData = ListItemData.build {
        identifier = "progress"
        type = ListItemData.PROGRESS
    }

    suspend fun processIntent(intent: ViewIntent) = intentFlow.emit(intent)

    override fun onCleared() {
        splitTunneling.persist()
        super.onCleared()
    }
}
