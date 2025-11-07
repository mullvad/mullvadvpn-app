package net.mullvad.mullvadvpn

import android.app.Application
import androidx.compose.runtime.Composer
import androidx.compose.runtime.ExperimentalComposeRuntimeApi
import co.touchlab.kermit.Logger
import co.touchlab.kermit.Severity
import java.io.IOException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.di.ApplicationScope
import net.mullvad.mullvadvpn.di.appModule
import net.mullvad.mullvadvpn.service.notifications.NotificationChannelFactory
import net.mullvad.mullvadvpn.service.notifications.NotificationManager
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.AccountExpiryNotificationProvider
import net.mullvad.mullvadvpn.usecase.AccountExpiryNotificationActionUseCase
import net.mullvad.mullvadvpn.usecase.NotificationAction
import net.mullvad.mullvadvpn.usecase.ScheduleNotificationAlarmUseCase
import net.mullvad.mullvadvpn.util.FileLogWriter
import org.koin.android.ext.android.getKoin
import org.koin.android.ext.koin.androidContext
import org.koin.core.context.loadKoinModules
import org.koin.core.context.startKoin

private const val LOG_TAG = "mullvad"

@OptIn(ExperimentalComposeRuntimeApi::class)
class MullvadApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        Logger.setTag(LOG_TAG)
        if (!BuildConfig.DEBUG) {
            Logger.setMinSeverity(Severity.Info)
        }
        // Improve compose stack traces
        // Comes with a performance penalty, so only enable in debug builds
        Composer.setDiagnosticStackTraceEnabled(BuildConfig.DEBUG)
        startKoin { androidContext(this@MullvadApplication) }
        loadKoinModules(listOf(appModule))
        with(getKoin()) {
            get<NotificationChannelFactory>()
            get<NotificationManager>()
            initFileLogger(get<ApplicationScope>())

            handleAccountExpiry(
                scope = get<ApplicationScope>(),
                accountExpiryUseCase = get<AccountExpiryNotificationActionUseCase>(),
                scheduleNotificationAlarmUseCase = get<ScheduleNotificationAlarmUseCase>(),
                accountExpiryNotificationProvider = get<AccountExpiryNotificationProvider>(),
            )
        }
    }

    private fun initFileLogger(scope: CoroutineScope) {
        try {
            val fileLogWriter =
                FileLogWriter(logDir = this.filesDir.toPath().resolve("app_logs"), scope = scope)
            Logger.addLogWriter(fileLogWriter)
        } catch (e: IOException) { // This shouldn't happen but just in case catch here.
            Logger.e("Failed to initialize file log writer", e)
        }
    }

    private fun handleAccountExpiry(
        scope: CoroutineScope,
        accountExpiryUseCase: AccountExpiryNotificationActionUseCase,
        scheduleNotificationAlarmUseCase: ScheduleNotificationAlarmUseCase,
        accountExpiryNotificationProvider: AccountExpiryNotificationProvider,
    ) {
        scope.launch {
            accountExpiryUseCase().collect { action ->
                when (action) {
                    NotificationAction.CancelExisting -> {
                        accountExpiryNotificationProvider.cancelNotification()
                        scheduleNotificationAlarmUseCase(accountExpiry = null)
                    }

                    is NotificationAction.ScheduleAlarm ->
                        scheduleNotificationAlarmUseCase(accountExpiry = action.alarmTime)
                }
            }
        }
    }
}
