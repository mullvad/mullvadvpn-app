package net.mullvad.mullvadvpn

import android.app.Application
import org.koin.android.ext.koin.androidContext
import org.koin.core.context.startKoin

/**
 * In Android, separate instances of the application class (MullvadApplication) will be instantiated
 * for each process. That also means that a only common logic should be placed here.
 */
class MullvadApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        // Used to create/start separate DI graphs for each process. Avoid non-common classes etc.
        startKoin { androidContext(this@MullvadApplication) }
    }
}
