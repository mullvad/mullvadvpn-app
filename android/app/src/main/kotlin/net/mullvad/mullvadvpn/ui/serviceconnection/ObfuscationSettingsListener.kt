package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.ObfuscationSettings
import net.mullvad.mullvadvpn.model.SelectedObfuscation
import net.mullvad.mullvadvpn.model.Udp2TcpObfuscationSettings
import net.mullvad.talpid.util.EventNotifier

class ObfuscationSettingsListener(
    private val connection: Messenger,
    private val settingsListener: SettingsListener
) {
    private var obfuscationSettings: ObfuscationSettings? = null
    val onSelectedObfuscationChanged = EventNotifier(SelectedObfuscation.Off) // Default: Off

    fun select(obfuscation: SelectedObfuscation) {
        val settings = ObfuscationSettings(obfuscation, Udp2TcpObfuscationSettings(Constraint.Any()))
        connection.send(Request.SetObfuscationSettings(settings).message)
    }

    init {
        settingsListener.obfuscationSettingsNotifier.subscribe(this) { maybeObfuscationSettings ->
            maybeObfuscationSettings?.also { obfuscationSettings ->
                synchronized(this) {
                    this.obfuscationSettings = obfuscationSettings
                    onSelectedObfuscationChanged.notifyIfChanged(obfuscationSettings.selectedObfuscation)
                }
            }
        }
    }

    fun onDestroy() {
        onSelectedObfuscationChanged.unsubscribeAll()

        settingsListener.obfuscationSettingsNotifier.unsubscribe(this)
    }
}
