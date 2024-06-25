package net.mullvad.mullvadvpn

import android.app.Application
import co.touchlab.kermit.Logger
import co.touchlab.kermit.Severity
import net.mullvad.mullvadvpn.di.appModule
import org.koin.android.ext.koin.androidContext
import org.koin.core.context.loadKoinModules
import org.koin.core.context.startKoin

private const val LOG_TAG = "mullvad"

/**
 * In Android, separate instances of the application class (MullvadApplication) will be instantiated
 * for each process. That also means that a only common logic should be placed here.
 */
class MullvadApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        Logger.setTag(LOG_TAG)
        if (!BuildConfig.DEBUG) {
            Logger.setMinSeverity(Severity.Info)
        }
        startKoin { androidContext(this@MullvadApplication) }
        loadKoinModules(listOf(appModule))
    }
}
