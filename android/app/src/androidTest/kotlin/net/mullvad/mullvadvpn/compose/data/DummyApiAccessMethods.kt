package net.mullvad.mullvadvpn.compose.data

import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodType

private const val UUID1 = "12345678-1234-5678-1234-567812345678"

val DIRECT_ACCESS_METHOD =
    ApiAccessMethod(
        id = ApiAccessMethodId.fromString(UUID1),
        name = ApiAccessMethodName.fromString("Direct"),
        enabled = true,
        apiAccessMethodType = ApiAccessMethodType.Direct
    )
