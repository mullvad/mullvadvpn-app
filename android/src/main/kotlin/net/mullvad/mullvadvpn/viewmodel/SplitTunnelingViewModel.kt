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
import net.mullvad.mullvadvpn.applist.AppInfo2
import net.mullvad.mullvadvpn.applist.ApplicationsProvider
import net.mullvad.mullvadvpn.applist.ViewIntent
import net.mullvad.mullvadvpn.model.ListItemData
import net.mullvad.mullvadvpn.model.WidgetState
import net.mullvad.mullvadvpn.service.SplitTunneling

class SplitTunnelingViewModel(
    private val appsProvider: ApplicationsProvider,
    private val splitTunneling: SplitTunneling
) : ViewModel() {
    private val _data = MutableSharedFlow<List<ListItemData>>(replay = 1)
    val data = _data.asSharedFlow() // read-only public view

    private val _intentFlow = MutableSharedFlow<ViewIntent>()

    private lateinit var excludedApps: MutableMap<String, AppInfo2>
    private lateinit var allAps: MutableMap<String, AppInfo2>

    private val defaultListItems = mutableListOf(
        createTextItem(R.string.split_tunneling_description)
        // We will have search item in future
    )

    init {
        viewModelScope.launch(Dispatchers.Default) {
            _intentFlow.shareIn(viewModelScope, SharingStarted.WhileSubscribed())
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

        viewModelScope.launch(Dispatchers.Default) {
            _data.emit(defaultListItems + createDivider(0) + createProgressItem())
        }
        fetchData()
        viewModelScope.launch(Dispatchers.Default) {
            if (!splitTunneling.enabled)
                splitTunneling.enabled = true
        }
    }

    private fun removeFromExcluded(packageName: String) {
        excludedApps.apply {
            get(packageName)?.let { appInfo ->
                allAps[packageName] = appInfo
                viewModelScope.launch {
                    splitTunneling.includeApp(packageName)
                }
            }
            remove(packageName)
        }
    }

    private fun addToExcluded(packageName: String) {
        allAps.apply {
            get(packageName)?.let { appInfo ->
                excludedApps[packageName] = appInfo
                viewModelScope.launch {
                    splitTunneling.excludeApp(packageName)
                }
            }
            remove(packageName)
        }
    }

    private fun fetchData() {
        viewModelScope.launch(Dispatchers.Default) {
            appsProvider.getAppsList().map { it.packageName to it }.toMap().let { applications ->
                val excludedPackages = applications.keys.intersect(splitTunneling.excludedAppList)
                // TODO: remove potential package names from splitTunneling list if they already uninstalled or filtered
                excludedApps = applications
                    .filterKeys { excludedPackages.contains(it) }
                    .toMutableMap()
                allAps = applications
                    .filterKeys { applications.keys.subtract(excludedPackages).contains(it) }
                    .toMutableMap()
                delay(100)
                publishList()
            }
        }
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

        if (allAps.isNotEmpty()) {
            listItems.add(createDivider(1))
            listItems.add(createMainItem(R.string.all_applications))
            listItems.addAll(
                allAps.values.sortedBy { it.name }.map { info ->
                    createApplicationItem(info.packageName, info.name, info.iconRes, false)
                }
            )
        }
        _data.emit(listItems)
    }

    private fun createApplicationItem(
        id: String,
        name: String,
        @DrawableRes icon: Int,
        checked: Boolean
    ): ListItemData = ListItemData.Builder()
        .setIdentifier(id)
        .setType(ListItemData.APPLICATION)
        .setText(name)
        .setIconRes(icon)
        .setAction(ListItemData.ItemAction(id))
        .setWidget(
            WidgetState.ImageState(
                if (checked) R.drawable.ic_icons_remove else R.drawable.ic_icons_add
            )
        )
        .build()

    private fun createDivider(id: Int): ListItemData = ListItemData.Builder()
        .setIdentifier("space_$id")
        .setType(ListItemData.DIVIDER)
        .build()

    private fun createMainItem(@StringRes text: Int): ListItemData = ListItemData.Builder()
        .setIdentifier("header_$text")
        .setType(ListItemData.ACTION)
        .setTextRes(text)
        .build()

    private fun createTextItem(@StringRes text: Int): ListItemData = ListItemData.Builder()
        .setIdentifier("text_$text")
        .setType(ListItemData.PLAIN)
        .setTextRes(text)
        .setAction(ListItemData.ItemAction(text.toString()))
        .build()

    private fun createProgressItem(): ListItemData = ListItemData.Builder()
        .setIdentifier("progress")
        .setType(ListItemData.PROGRESS)
        .build()

    suspend fun processIntent(intent: ViewIntent) = _intentFlow.emit(intent)

    override fun onCleared() {
        splitTunneling.persist()
        super.onCleared()
    }

    // Represents different states for the LatestNews screen
    sealed class LatestNewsUiState {
        data class Success(val news: List<ListItemData>) : LatestNewsUiState()
        data class Error(val exception: Throwable) : LatestNewsUiState()
    }
}
