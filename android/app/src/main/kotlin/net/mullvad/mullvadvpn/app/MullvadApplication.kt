package net.mullvad.mullvadvpn.app

import android.app.Application
import android.os.StrictMode
import android.os.strictmode.UntaggedSocketViolation
import androidx.compose.runtime.Composer
import androidx.compose.runtime.ExperimentalComposeRuntimeApi
import androidx.compose.runtime.tooling.ComposeStackTraceMode
import co.touchlab.kermit.Logger
import co.touchlab.kermit.Severity
import java.io.IOException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.asExecutor
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.app.util.FileLogWriter
import net.mullvad.mullvadvpn.di.ApplicationScope
import net.mullvad.mullvadvpn.di.KERMIT_FILE_LOG_DIR_NAME
import net.mullvad.mullvadvpn.di.appModule
import net.mullvad.mullvadvpn.lib.pushnotification.NotificationChannelFactory
import net.mullvad.mullvadvpn.lib.pushnotification.NotificationManager
import net.mullvad.mullvadvpn.lib.pushnotification.ScheduleNotificationAlarmUseCase
import net.mullvad.mullvadvpn.lib.pushnotification.accountexpiry.AccountExpiryNotificationProvider
import net.mullvad.mullvadvpn.lib.usecase.AccountExpiryNotificationActionUseCase
import net.mullvad.mullvadvpn.lib.usecase.NotificationAction
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
        if (BuildConfig.DEBUG) {
            // Improve compose stack traces
            // Comes with a performance penalty, so only enable in debug builds
            Composer.setDiagnosticStackTraceMode(ComposeStackTraceMode.SourceInformation)
            enableStrictMode()
        }
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
        scope.launch(Dispatchers.IO) {
            try {
                val fileLogWriter =
                    FileLogWriter(
                        logDir = filesDir.toPath().resolve(KERMIT_FILE_LOG_DIR_NAME),
                        scope = scope,
                    )
                Logger.addLogWriter(fileLogWriter)
            } catch (e: IOException) { // This shouldn't happen but just in case catch here.
                Logger.e("Failed to initialize file log writer", e)
            }
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

    private fun enableStrictMode() {
        val executor = Dispatchers.Default.asExecutor()

        StrictMode.setThreadPolicy(
            StrictMode.ThreadPolicy.Builder()
                .detectAll()
                .penaltyListener(executor) { violation ->
                    // It is a known issue that MullvadVpnService performs IO on the UI thread,
                    // but we have chosen to keep it that way for now. See: DROID-2486
                    val ignore =
                        violation.stackTrace.any {
                            it.className == "net.mullvad.mullvadvpn.app.service.MullvadVpnService"
                        }
                    if (ignore) {
                        return@penaltyListener
                    }
                    android.util.Log.e(
                        "StrictMode",
                        "StrictMode thread policy violation:",
                        violation,
                    )
                }
                .build()
        )

        StrictMode.setVmPolicy(
            StrictMode.VmPolicy.Builder()
                .detectAll()
                .penaltyListener(executor) { violation ->
                    // Filter out violations that we don't care about that would spam the logs
                    if (violation is UntaggedSocketViolation) {
                        return@penaltyListener
                    }
                    android.util.Log.e("StrictMode", "StrictMode VM policy violation:", violation)
                }
                .build()
        )
    }
}
