package net.mullvad.mullvadvpn.compose.preview

import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodSetting
import net.mullvad.mullvadvpn.lib.model.Cipher
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.SocksAuth

private const val UUID1 = "12345678-1234-5678-1234-567812345678"
private const val UUID2 = "12345678-1234-5678-1234-567812345679"
private const val UUID3 = "12345678-1234-5678-1234-567812345671"
private const val UUID4 = "12345678-1234-5678-1234-567812345672"

internal val defaultAccessMethods =
    listOf(
        ApiAccessMethodSetting(
            id = ApiAccessMethodId.fromString(UUID1),
            name = ApiAccessMethodName.fromString("Direct"),
            enabled = true,
            apiAccessMethod = ApiAccessMethod.Direct,
        ),
        ApiAccessMethodSetting(
            id = ApiAccessMethodId.fromString(UUID2),
            name = ApiAccessMethodName.fromString("Bridges"),
            enabled = false,
            apiAccessMethod = ApiAccessMethod.Bridges,
        ),
    )

internal val socks5Remote =
    ApiAccessMethodSetting(
        id = ApiAccessMethodId.fromString(UUID3),
        name = ApiAccessMethodName.fromString("Socks5 Remote"),
        enabled = true,
        apiAccessMethod =
            ApiAccessMethod.CustomProxy.Socks5Remote(
                ip = "192.167.1.1",
                port = Port(80),
                auth = SocksAuth(username = "hej", password = "password"),
            ),
    )

internal val shadowsocks =
    ApiAccessMethodSetting(
        ApiAccessMethodId.fromString(UUID4),
        ApiAccessMethodName.fromString("ShadowSocks"),
        enabled = true,
        ApiAccessMethod.CustomProxy.Shadowsocks(
            ip = "192.168.1.1",
            port = Port(123),
            password = "Password",
            cipher = Cipher.fromString("aes-128-cfb"),
        ),
    )
