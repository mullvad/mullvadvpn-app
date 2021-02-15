package net.mullvad.mullvadvpn.di

import android.content.Context
import android.content.pm.PackageManager
import net.mullvad.mullvadvpn.applist.ApplicationsIconManager
import net.mullvad.mullvadvpn.applist.ApplicationsProvider
import net.mullvad.mullvadvpn.service.SplitTunneling
import net.mullvad.mullvadvpn.viewmodel.SplitTunnelingViewModel
import org.koin.androidx.viewmodel.dsl.viewModel
import org.koin.core.qualifier.named
import org.koin.dsl.module
import org.koin.dsl.onClose

val appModule = module {

    single { SplitTunneling(get()) }
    single<PackageManager> { get<Context>().packageManager }
    single<String> (named(SELF_PACKAGE)) { get<Context>().packageName }

    scope(named(APPS_SCOPE)) {
        viewModel { SplitTunnelingViewModel(get(), get()) }
        scoped { ApplicationsIconManager(get<PackageManager>()) } onClose { it?.dispose() }
        scoped { ApplicationsProvider(get<PackageManager>(), get<String>(named(SELF_PACKAGE))) }
    }
}
const val APPS_SCOPE = "APPS_SCOPE"
const val SELF_PACKAGE = "SELF_PACKAGE"
