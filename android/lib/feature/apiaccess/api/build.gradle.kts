plugins { alias(libs.plugins.mullvad.android.library.feature.api) }

android { namespace = "net.mullvad.mullvadvpn.feature.apiaccess.api" }

dependencies { implementation(projects.lib.model) }
