package net.mullvad.mullvadvpn.applist

import android.content.Context
import android.support.v7.widget.RecyclerView.Adapter
import android.view.LayoutInflater
import android.view.ViewGroup
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.util.JobTracker

class AppListAdapter(context: Context) : Adapter<AppListItemHolder>() {
    private val appList = ArrayList<AppInfo>()
    private val jobTracker = JobTracker()
    private val packageManager = context.packageManager

    init {
        jobTracker.newBackgroundJob("populateAppList") {
            populateAppList(context)
        }
    }

    override fun getItemCount() = appList.size

    override fun onCreateViewHolder(parentView: ViewGroup, type: Int): AppListItemHolder {
        val inflater = LayoutInflater.from(parentView.context)
        val view = inflater.inflate(R.layout.app_list_item, parentView, false)

        return AppListItemHolder(packageManager, jobTracker, view)
    }

    override fun onBindViewHolder(holder: AppListItemHolder, position: Int) {
        holder.appInfo = appList.get(position)
    }

    private fun populateAppList(context: Context) {
        val applications = packageManager
            .getInstalledApplications(0)
            .map { info -> AppInfo(info, packageManager.getApplicationLabel(info).toString()) }

        appList.apply {
            clear()
            addAll(applications)
            sortBy { info -> info.label }
        }

        jobTracker.newUiJob("notifyAppListChanges") {
            notifyItemRangeInserted(0, applications.size)
        }
    }
}
