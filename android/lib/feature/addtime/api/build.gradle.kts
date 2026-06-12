plugins { alias(libs.plugins.mullvad.android.library.feature.api) }

android { namespace = "net.mullvad.mullvadvpn.feature.addtime.api" }

dependencies { implementation(projects.lib.payment) }
