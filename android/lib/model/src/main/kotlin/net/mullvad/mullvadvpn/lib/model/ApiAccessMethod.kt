package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed interface ApiAccessMethod : Parcelable {
    @Parcelize data object Direct : ApiAccessMethod

    @Parcelize data object Bridges : ApiAccessMethod

    @Parcelize data object EncryptedDns : ApiAccessMethod

    sealed interface CustomProxy : ApiAccessMethod {
        @Parcelize
        data class Socks5Remote(val ip: String, val port: Port, val auth: SocksAuth?) : CustomProxy

        @Parcelize
        data class Shadowsocks(
            val ip: String,
            val port: Port,
            val password: String?,
            val cipher: Cipher,
        ) : CustomProxy
    }
}
