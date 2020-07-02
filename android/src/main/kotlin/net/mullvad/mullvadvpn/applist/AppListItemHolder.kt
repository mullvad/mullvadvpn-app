package net.mullvad.mullvadvpn.applist

import android.content.pm.ApplicationInfo
import android.content.pm.PackageManager
import android.support.v7.widget.RecyclerView.ViewHolder
import android.view.View
import android.widget.ImageView
import android.widget.TextView
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.CellSwitch

class AppListItemHolder(private val packageManager: PackageManager, view: View) : ViewHolder(view) {
    private val icon: ImageView = view.findViewById(R.id.icon)
    private val name: TextView = view.findViewById(R.id.name)
    private val excluded: CellSwitch = view.findViewById(R.id.excluded)

    var appInfo by observable<ApplicationInfo?>(null) { _, _, info ->
        if (info != null) {
            name.text = packageManager.getApplicationLabel(info)
        } else {
            name.text = ""
        }
    }
}
