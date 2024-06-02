package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodInvalidDataErrors
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.lib.model.Cipher
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.TransportProtocol

sealed interface EditApiAccessMethodUiState {
    val editMode: Boolean

    data class Loading(override val editMode: Boolean) : EditApiAccessMethodUiState

    data class Content(
        override val editMode: Boolean,
        val formData: EditApiAccessFormData,
        val formErrors: ApiAccessMethodInvalidDataErrors?,
        val testMethodState: TestMethodState?
    ) : EditApiAccessMethodUiState
}

sealed interface EditApiAccessFormData {
    val name: ApiAccessMethodName?

    data class Socks5Local(
        override val name: ApiAccessMethodName? = null,
        val remoteIp: String? = null,
        val remotePort: Port? = null,
        val remoteTransportProtocol: TransportProtocol,
        val localPort: Port? = null
    ) : EditApiAccessFormData

    data class Socks5Remote(
        override val name: ApiAccessMethodName? = null,
        val ip: String? = null,
        val port: Port? = null,
        val enableAuthentication: Boolean,
        val username: String? = null,
        val password: String? = null
    ) : EditApiAccessFormData

    data class Shadowsocks(
        override val name: ApiAccessMethodName? = null,
        val ip: String? = null,
        val port: Port? = null,
        val password: String? = null,
        val cipher: Cipher
    ) : EditApiAccessFormData

    fun updateName(name: ApiAccessMethodName) =
        when (this) {
            is Shadowsocks -> copy(name = name)
            is Socks5Local -> copy(name = name)
            is Socks5Remote -> copy(name = name)
        }

    fun updateServerIp(ip: String) =
        when (this) {
            is Shadowsocks -> copy(ip = ip)
            is Socks5Local -> copy(remoteIp = ip)
            is Socks5Remote -> copy(ip = ip)
        }

    fun updateRemotePort(port: Port?) =
        when (this) {
            is Shadowsocks -> copy(port = port)
            is Socks5Local -> copy(remotePort = port)
            is Socks5Remote -> copy(port = port)
        }

    fun updateLocalPort(port: Port?) =
        when (this) {
            is Socks5Local -> copy(localPort = port)
            is Shadowsocks,
            is Socks5Remote -> error("$this does not have local port")
        }

    fun updatePassword(password: String) =
        when (this) {
            is Socks5Local -> error("$this does not have password")
            is Shadowsocks -> copy(password = password)
            is Socks5Remote -> copy(password = password)
        }

    fun updateCipher(cipher: Cipher) =
        when (this) {
            is Socks5Local,
            is Socks5Remote -> error("$this does not have cipher")
            is Shadowsocks -> copy(cipher = cipher)
        }

    fun updateAuthenticationEnabled(enableAuthentication: Boolean) =
        when (this) {
            is Socks5Local,
            is Shadowsocks -> error("$this does not have enable authentication")
            is Socks5Remote -> copy(enableAuthentication = enableAuthentication)
        }

    fun updateUsername(username: String) =
        when (this) {
            is Socks5Local,
            is Shadowsocks -> error("$this does not have username")
            is Socks5Remote -> copy(username = username)
        }

    fun updateTransportProtocol(transportProtocol: TransportProtocol) =
        when (this) {
            is Socks5Local -> copy(remoteTransportProtocol = transportProtocol)
            is Shadowsocks,
            is Socks5Remote -> error("$this does not have transport protocol")
        }

    companion object {
        // Default values
        fun empty() = Shadowsocks(cipher = Cipher.first())

        fun emptyFromTypeAndName(type: ApiAccessMethodTypes, name: ApiAccessMethodName?) =
            when (type) {
                ApiAccessMethodTypes.SHADOWSOCKS ->
                    Shadowsocks(name = name, cipher = Cipher.first())
                ApiAccessMethodTypes.SOCKS5_LOCAL ->
                    Socks5Local(name = name, remoteTransportProtocol = TransportProtocol.Tcp)
                ApiAccessMethodTypes.SOCKS5_REMOTE ->
                    Socks5Remote(name = name, enableAuthentication = false)
            }
    }
}

enum class ApiAccessMethodTypes {
    SOCKS5_LOCAL,
    SOCKS5_REMOTE,
    SHADOWSOCKS
}

sealed interface TestMethodState {
    data object Testing : TestMethodState
    data object Successful : TestMethodState

    data object Failed : TestMethodState
}
