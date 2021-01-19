package net.mullvad.mullvadvpn.applist

import android.content.pm.PackageManager
import android.graphics.drawable.Drawable
import android.view.View
import android.widget.ImageView
import android.widget.TextView
import androidx.recyclerview.widget.RecyclerView.ViewHolder
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.service.SplitTunneling
import net.mullvad.mullvadvpn.ui.widget.CellSwitch
import net.mullvad.mullvadvpn.util.JobTracker

class AppListItemHolder(
    private val splitTunneling: SplitTunneling,
    private val packageManager: PackageManager,
    private val jobTracker: JobTracker,
    view: View
) : ViewHolder(view) {
    private val loading: View = view.findViewById(R.id.loading)
    private val icon: ImageView = view.findViewById(R.id.icon)
    private val name: TextView = view.findViewById(R.id.name)
    private val excluded: CellSwitch = view.findViewById(R.id.excluded)

    var appInfo by observable<AppInfo?>(null) { _, _, info ->
        if (info != null) {
            val iconImage = info.icon

            name.text = info.label

            if (iconImage != null) {
                showIcon(iconImage)
            } else {
                hideIcon()
                loadIcon(info)
            }

            if (splitTunneling.isAppExcluded(info.info.packageName)) {
                excluded.forcefullySetState(CellSwitch.State.ON)
            } else {
                excluded.forcefullySetState(CellSwitch.State.OFF)
            }
        } else {
            name.text = ""
            hideIcon()
        }
    }

    init {
        view.setOnClickListener {
            excluded.toggle()
        }

        excluded.listener = { state ->
            appInfo?.info?.packageName?.let { app ->
                when (state) {
                    CellSwitch.State.ON -> splitTunneling.excludeApp(app)
                    CellSwitch.State.OFF -> splitTunneling.includeApp(app)
                }
            }
        }
    }

    private fun hideIcon() {
        icon.visibility = View.GONE
        loading.visibility = View.VISIBLE
    }

    private fun showIcon(iconImage: Drawable) {
        loading.visibility = View.GONE
        icon.setImageDrawable(iconImage)
        icon.visibility = View.VISIBLE
    }

    private fun loadIcon(info: AppInfo) {
        jobTracker.newUiJob("load icon for ${info.info.packageName}") {
            val iconImage = jobTracker.runOnBackground {
                packageManager.getApplicationIcon(info.info)
            }

            info.icon = iconImage

            showIcon(iconImage)
        }
    }
}
