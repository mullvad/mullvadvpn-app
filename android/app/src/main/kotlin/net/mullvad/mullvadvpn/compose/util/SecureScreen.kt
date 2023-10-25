package net.mullvad.mullvadvpn.compose.util

import android.app.Activity
import android.view.WindowManager
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.platform.LocalContext
import net.mullvad.mullvadvpn.BuildConfig

@Composable
fun SecureScreenWhileInView() {
    if (BuildConfig.DEBUG) {
        return
    }
    val context = LocalContext.current
    val window = (context as Activity).window
    val secureScreenWasEnabled = rememberSaveable {
        window.attributes.flags and WindowManager.LayoutParams.FLAG_SECURE != 0
    }

    DisposableEffect(Unit) {
        window.addFlags(WindowManager.LayoutParams.FLAG_SECURE)
        onDispose {
            if (!secureScreenWasEnabled) {
                window.clearFlags(WindowManager.LayoutParams.FLAG_SECURE)
            }
        }
    }
}
