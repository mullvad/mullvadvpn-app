plugins { alias(libs.plugins.mullvad.android.library.feature.api) }

android { namespace = "net.mullvad.mullvadvpn.feature.customlist.api" }

dependencies {
    implementation(projects.lib.model)
}
