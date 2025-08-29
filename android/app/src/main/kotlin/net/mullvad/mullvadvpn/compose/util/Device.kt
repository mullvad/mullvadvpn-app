package net.mullvad.mullvadvpn.compose.util

import android.content.pm.PackageManager
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.booleanResource
import net.mullvad.mullvadvpn.R

@Composable
fun isTv(): Boolean {
    return booleanResource(R.bool.isTv) ||
        LocalContext.current.packageManager.hasSystemFeature(PackageManager.FEATURE_LEANBACK)
}
