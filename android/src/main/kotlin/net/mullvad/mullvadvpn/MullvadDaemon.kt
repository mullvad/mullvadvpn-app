package net.mullvad.mullvadvpn

import net.mullvad.mullvadvpn.model.AccountData

class MullvadDaemon {
    init {
        System.loadLibrary("mullvad_jni")
        initialize()
    }

    external fun getAccountData(accountToken: String): AccountData?
    external fun setAccount(accountToken: String?)

    private external fun initialize()
}
