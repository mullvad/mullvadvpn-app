package net.mullvad.mullvadvpn

import android.app.Application
import net.mullvad.mullvadvpn.di.appModule
import org.koin.android.ext.koin.androidContext
import org.koin.core.context.startKoin

class MullvadApplication : Application() {

    override fun onCreate() {
        super.onCreate()
        // start Koin!
        startKoin {
            // declare used Android context
            androidContext(this@MullvadApplication)
            // declare modules
            modules(appModule)
        }
    }
}
