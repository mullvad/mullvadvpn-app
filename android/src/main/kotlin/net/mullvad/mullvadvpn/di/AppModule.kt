package net.mullvad.mullvadvpn.di

import android.content.pm.PackageManager
import kotlinx.coroutines.Dispatchers
import net.mullvad.mullvadvpn.applist.ApplicationsIconManager
import net.mullvad.mullvadvpn.applist.ApplicationsProvider
import net.mullvad.mullvadvpn.service.SplitTunneling
import net.mullvad.mullvadvpn.viewmodel.SplitTunnelingViewModel
import org.koin.android.ext.koin.androidContext
import org.koin.androidx.viewmodel.dsl.viewModel
import org.koin.core.qualifier.named
import org.koin.dsl.module
import org.koin.dsl.onClose

val appModule = module {

    single<SplitTunneling> { SplitTunneling(androidContext()) }
    single<PackageManager> { androidContext().packageManager }
    single<String> (named(SELF_PACKAGE_NAME)) { androidContext().packageName }

    scope(named(APPS_SCOPE)) {
        viewModel { SplitTunnelingViewModel(get(), get(), Dispatchers.Default) }
        scoped { ApplicationsIconManager(get()) } onClose { it?.dispose() }
        scoped { ApplicationsProvider(get(), get(named(SELF_PACKAGE_NAME))) }
    }
}
const val APPS_SCOPE = "APPS_SCOPE"
const val SELF_PACKAGE_NAME = "SELF_PACKAGE_NAME"
