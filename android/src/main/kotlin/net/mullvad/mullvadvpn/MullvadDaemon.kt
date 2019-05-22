package net.mullvad.mullvadvpn

import net.mullvad.mullvadvpn.model.AccountData
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.RelaySettingsUpdate
import net.mullvad.mullvadvpn.model.Settings

class MullvadDaemon {
    init {
        System.loadLibrary("mullvad_jni")
        initialize()
    }

    external fun getAccountData(accountToken: String): AccountData?
    external fun getRelayLocations(): RelayList
    external fun getSettings(): Settings
    external fun setAccount(accountToken: String?)
    external fun updateRelaySettings(update: RelaySettingsUpdate)

    private external fun initialize()
}
