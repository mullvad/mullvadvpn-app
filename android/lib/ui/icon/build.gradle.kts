plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.kotlin.parcelize)
}

android { namespace = "net.mullvad.mullvadvpn.lib.ui.icon" }

dependencies { implementation(libs.compose.ui) }
