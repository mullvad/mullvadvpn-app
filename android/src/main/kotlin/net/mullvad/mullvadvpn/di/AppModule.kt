package net.mullvad.mullvadvpn.di

import android.content.Context
import android.content.pm.PackageManager
import net.mullvad.mullvadvpn.applist.ApplicationsProvider
import net.mullvad.mullvadvpn.service.SplitTunneling
import net.mullvad.mullvadvpn.viewmodel.SplitTunnelingViewModel
import org.koin.androidx.viewmodel.dsl.viewModel
import org.koin.core.qualifier.named
import org.koin.dsl.module

val appModule = module {

    single<SplitTunneling> { SplitTunneling(get()) }

    single<ApplicationsProvider> { ApplicationsProvider(get<PackageManager>(), get<String>(named("packagename"))) }

    single<PackageManager> { get<Context>().packageManager }

    single<String> (named("packagename")) { get<Context>().packageName }

    viewModel { SplitTunnelingViewModel(get(), get()) }
}
