package net.mullvad.mullvadvpn

import net.mullvad.mullvadvpn.model.AccountData
import net.mullvad.mullvadvpn.model.Settings

class MullvadDaemon {
    init {
        System.loadLibrary("mullvad_jni")
        initialize()
    }

    external fun getAccountData(accountToken: String): AccountData?
    external fun getSettings(): Settings
    external fun setAccount(accountToken: String?)

    private external fun initialize()
}
