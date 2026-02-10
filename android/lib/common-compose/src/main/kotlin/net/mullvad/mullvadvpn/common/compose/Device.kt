package net.mullvad.mullvadvpn.common.compose

import android.content.pm.PackageManager
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.booleanResource
import net.mullvad.mullvadvpn.lib.ui.resource.R

@Composable
fun isTv(): Boolean {
    return booleanResource(R.bool.isTv) ||
        LocalContext.current.packageManager.hasSystemFeature(PackageManager.FEATURE_LEANBACK)
}
