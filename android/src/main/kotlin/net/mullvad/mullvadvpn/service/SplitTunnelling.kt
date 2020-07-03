package net.mullvad.mullvadvpn.service

import kotlin.properties.Delegates.observable

class SplitTunnelling {
    private val excludedApps = HashSet<String>()

    val excludedAppList
        get() = if (enabled) {
            excludedApps.toList()
        } else {
            emptyList()
        }

    var enabled by observable(false) { _, _, _ -> update() }
    var onChange: ((List<String>) -> Unit)? = null

    fun isAppExcluded(appPackageName: String) = excludedApps.contains(appPackageName)

    fun excludeApp(appPackageName: String) {
        excludedApps.add(appPackageName)
        update()
    }

    fun includeApp(appPackageName: String) {
        excludedApps.remove(appPackageName)
        update()
    }

    private fun update() {
        onChange?.invoke(excludedAppList)
    }
}
