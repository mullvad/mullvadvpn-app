package net.mullvad.mullvadvpn.feature.apiaccess.impl.util

import android.content.res.Resources
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodSetting
import net.mullvad.mullvadvpn.lib.ui.resource.R

@Composable
fun ApiAccessMethodSetting?.toDisplayName() =
    when (this?.apiAccessMethod) {
        ApiAccessMethod.Direct -> stringResource(R.string.direct)
        ApiAccessMethod.Bridges,
        ApiAccessMethod.EncryptedDns,
        is ApiAccessMethod.CustomProxy -> this.name.toString()
        null -> "-"
    }

fun ApiAccessMethodSetting.toDisplayName(resources: Resources) =
    when (this.apiAccessMethod) {
        ApiAccessMethod.Direct -> resources.getString(R.string.direct)
        ApiAccessMethod.Bridges,
        ApiAccessMethod.EncryptedDns,
        is ApiAccessMethod.CustomProxy -> this.name.toString()
    }
