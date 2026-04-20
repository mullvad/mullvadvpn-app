plugins {
    alias(libs.plugins.mullvad.android.library)
    alias(libs.plugins.mullvad.android.library.feature.impl)
    alias(libs.plugins.kotlin.android)
}

android { namespace = "net.mullvad.mullvadvpn.feature.applisting.impl" }

dependencies { implementation(projects.lib.feature.applisting.api) }
