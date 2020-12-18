package net.mullvad.mullvadvpn.viewmodel

import androidx.annotation.StringRes
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.applist.AppData
import net.mullvad.mullvadvpn.applist.ApplicationsProvider
import net.mullvad.mullvadvpn.applist.ViewIntent
import net.mullvad.mullvadvpn.model.ListItemData
import net.mullvad.mullvadvpn.model.WidgetState
import net.mullvad.mullvadvpn.service.endpoint.SplitTunneling

class SplitTunnelingViewModel(
    private val appsProvider: ApplicationsProvider,
    private val splitTunneling: SplitTunneling,
    dispatcher: CoroutineDispatcher
) : ViewModel() {
    private val listItemsSink = MutableSharedFlow<List<ListItemData>>(replay = 1)
    // read-only public view
    val listItems: SharedFlow<List<ListItemData>> = listItemsSink.asSharedFlow()

    private val intentFlow = MutableSharedFlow<ViewIntent>()
    private val isUIReady = CompletableDeferred<Unit>()
    private val excludedApps: MutableMap<String, AppData> = mutableMapOf()
    private val notExcludedApps: MutableMap<String, AppData> = mutableMapOf()

    private val defaultListItems: List<ListItemData> = listOf(
        createTextItem(R.string.split_tunneling_description)
        // We will have search item in future
    )

    init {
        viewModelScope.launch(dispatcher) {
            listItemsSink.emit(defaultListItems + createDivider(0) + createProgressItem())
            // this will be removed after changes on native to ignore enable parameter
            if (!splitTunneling.enabled)
                splitTunneling.enabled = true
            fetchData()
        }
        viewModelScope.launch(dispatcher) {
            intentFlow.shareIn(viewModelScope, SharingStarted.WhileSubscribed())
                .collect(::handleIntents)
        }
    }

    suspend fun processIntent(intent: ViewIntent) = intentFlow.emit(intent)

    override fun onCleared() {
        splitTunneling.persist()
        super.onCleared()
    }

    private suspend fun handleIntents(viewIntent: ViewIntent) {
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
            is ViewIntent.ViewIsReady -> isUIReady.complete(Unit)
        }
    }

    private fun removeFromExcluded(packageName: String) {
        excludedApps.remove(packageName)?.let { appInfo ->
            notExcludedApps[packageName] = appInfo
            splitTunneling.includeApp(packageName)
        }
    }

    private fun addToExcluded(packageName: String) {
        notExcludedApps.remove(packageName)?.let { appInfo ->
            excludedApps[packageName] = appInfo
            splitTunneling.excludeApp(packageName)
        }
    }

    private suspend fun fetchData() {
        appsProvider.getAppsList()
            .partition { app -> splitTunneling.excludedAppList?.contains(app.packageName) ?: false }
            .let { (excludedAppsList, notExcludedAppsList) ->
                // TODO: remove potential package names from splitTunneling list
                //       if they already uninstalled or filtered; but not in ViewModel
                excludedAppsList.map { it.packageName to it }.toMap(excludedApps)
                notExcludedAppsList.map { it.packageName to it }.toMap(notExcludedApps)
            }
        isUIReady.await()
        publishList()
    }

    private suspend fun publishList() {
        val listItems = ArrayList(defaultListItems)
        if (excludedApps.isNotEmpty()) {
            listItems += createDivider(0)
            listItems += createMainItem(R.string.exclude_applications)
            listItems += excludedApps.values.sortedBy { it.name }.map { info ->
                createApplicationItem(info, true)
            }
        }
        if (notExcludedApps.isNotEmpty()) {
            listItems += createDivider(1)
            listItems += createMainItem(R.string.all_applications)
            listItems += notExcludedApps.values.sortedBy { it.name }.map { info ->
                createApplicationItem(info, false)
            }
        }
        listItemsSink.emit(listItems)
    }

    private fun createApplicationItem(appData: AppData, checked: Boolean): ListItemData =
        ListItemData.build(appData.packageName) {
            type = ListItemData.APPLICATION
            text = appData.name
            iconRes = appData.iconRes
            action = ListItemData.ItemAction(appData.packageName)
            widget = WidgetState.ImageState(
                if (checked) R.drawable.ic_icons_remove else R.drawable.ic_icons_add
            )
        }

    private fun createDivider(id: Int): ListItemData = ListItemData.build("space_$id") {
        type = ListItemData.DIVIDER
    }

    private fun createMainItem(@StringRes text: Int): ListItemData =
        ListItemData.build("header_$text") {
            type = ListItemData.ACTION
            textRes = text
        }

    private fun createTextItem(@StringRes text: Int): ListItemData =
        ListItemData.build("text_$text") {
            type = ListItemData.PLAIN
            textRes = text
            action = ListItemData.ItemAction(text.toString())
        }

    private fun createProgressItem(): ListItemData = ListItemData.build(identifier = "progress") {
        type = ListItemData.PROGRESS
    }
}
