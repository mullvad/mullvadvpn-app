package net.mullvad.mullvadvpn.compose.util

import android.content.Context
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodSetting

@Composable
fun ApiAccessMethodSetting?.toDisplayName() =
    when (this?.apiAccessMethod) {
        ApiAccessMethod.Direct -> stringResource(R.string.direct)
        ApiAccessMethod.Bridges,
        ApiAccessMethod.EncryptedDns,
        is ApiAccessMethod.CustomProxy -> this.name.toString()
        null -> "-"
    }

fun ApiAccessMethodSetting.toDisplayName(context: Context) =
    when (this.apiAccessMethod) {
        ApiAccessMethod.Direct -> context.getString(R.string.direct)
        ApiAccessMethod.Bridges,
        ApiAccessMethod.EncryptedDns,
        is ApiAccessMethod.CustomProxy -> this.name.toString()
    }
