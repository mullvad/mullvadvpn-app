plugins {
    id("mullvad.android-library")
    alias(libs.plugins.kotlin.parcelize)
}

android { namespace = "net.mullvad.mullvadvpn.lib.endpoint" }

dependencies { implementation(libs.kotlin.stdlib) }
