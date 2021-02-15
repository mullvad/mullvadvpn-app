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
    single { ApplicationsProvider(get<PackageManager>(), get<String>(named("packagename"))) }
    single<PackageManager> { get<Context>().packageManager }
    single<String> (named("packagename")) { get<Context>().packageName }

    viewModel { SplitTunnelingViewModel(get(), get()) }

    scope(named(SPLITTUNNELING_SCOPE)) {
        scoped { ApplicationsIconManager(get<PackageManager>()) } onClose { it?.dispose() }
    }
}
const val SPLITTUNNELING_SCOPE = "SPLITTUNNELING_SCOPE"
