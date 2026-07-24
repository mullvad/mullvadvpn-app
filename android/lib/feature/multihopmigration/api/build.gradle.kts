plugins { alias(libs.plugins.mullvad.android.library.feature.api) }

android { namespace = "net.mullvad.mullvadvpn.feature.multihopmigration.api" }

dependencies { api(projects.lib.model) }
