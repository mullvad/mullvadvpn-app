package net.mullvad.mullvadvpn.applist

import android.Manifest
import android.content.Context
import android.content.pm.ApplicationInfo
import android.content.pm.PackageManager
import android.view.LayoutInflater
import android.view.ViewGroup
import androidx.recyclerview.widget.RecyclerView.Adapter
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.serviceconnection.SplitTunneling
import net.mullvad.mullvadvpn.util.JobTracker

class AppListAdapter(
    context: Context,
    private val splitTunneling: SplitTunneling
) : Adapter<AppListItemHolder>() {
    private val appList = ArrayList<AppInfo>()
    private val jobTracker = JobTracker()
    private val packageManager = context.packageManager
    private val thisPackageName = context.packageName

    private val applicationFilterPredicate: (ApplicationInfo) -> Boolean = { appInfo ->
        hasInternetPermission(appInfo.packageName) && !isSelfApplication(appInfo.packageName) &&
            isLaunchable(appInfo.packageName)
    }

    var onListReady: (suspend () -> Unit)? = null

    var isListReady = false
        private set

    var enabled by observable(false) { _, oldValue, newValue ->
        if (oldValue != newValue) {
            if (newValue == true) {
                notifyItemRangeInserted(0, appList.size)
            } else {
                notifyItemRangeRemoved(0, appList.size)
            }
        }
    }

    init {
        jobTracker.newBackgroundJob("populateAppList") {
            populateAppList()
        }
    }

    override fun getItemCount() = if (enabled) { appList.size } else { 0 }

    override fun onCreateViewHolder(parentView: ViewGroup, type: Int): AppListItemHolder {
        val inflater = LayoutInflater.from(parentView.context)
        val view = inflater.inflate(R.layout.app_list_item, parentView, false)

        return AppListItemHolder(splitTunneling, packageManager, jobTracker, view)
    }

    override fun onBindViewHolder(holder: AppListItemHolder, position: Int) {
        holder.appInfo = appList.get(position)
    }

    private fun populateAppList() {
        val applications = packageManager
            .getInstalledApplications(0)
            .filter(applicationFilterPredicate)
            .map { info -> AppInfo(info, packageManager.getApplicationLabel(info).toString()) }

        appList.apply {
            clear()
            addAll(applications)
            sortBy { info -> info.label }
        }

        jobTracker.newUiJob("notifyAppListChanges") {
            isListReady = true
            onListReady?.invoke()
            notifyItemRangeInserted(0, applications.size)
        }
    }

    private fun hasInternetPermission(packageName: String): Boolean {
        return PackageManager.PERMISSION_GRANTED ==
            packageManager.checkPermission(Manifest.permission.INTERNET, packageName)
    }

    private fun isSelfApplication(packageName: String): Boolean {
        return packageName == thisPackageName
    }

    private fun isLaunchable(packageName: String): Boolean {
        return packageManager.getLaunchIntentForPackage(packageName) != null
    }
}
