package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed interface ApiAccessMethodType : Parcelable {
    @Parcelize data object Direct : ApiAccessMethodType

    @Parcelize data object Bridges : ApiAccessMethodType

    sealed interface CustomProxy : ApiAccessMethodType {
        @Parcelize
        data class Socks5Remote(val ip: String, val port: Port, val auth: SocksAuth?) : CustomProxy

        @Parcelize
        data class Shadowsocks(
            val ip: String,
            val port: Port,
            val password: String?,
            val cipher: Cipher
        ) : CustomProxy
    }
}
