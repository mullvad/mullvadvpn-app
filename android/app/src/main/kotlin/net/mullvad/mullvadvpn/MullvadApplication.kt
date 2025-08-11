package net.mullvad.mullvadvpn

import android.app.Application
import co.touchlab.kermit.Logger
import co.touchlab.kermit.Severity
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.di.appModule
import net.mullvad.mullvadvpn.service.notifications.NotificationChannelFactory
import net.mullvad.mullvadvpn.service.notifications.NotificationManager
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.AccountExpiryNotificationProvider
import net.mullvad.mullvadvpn.usecase.AccountExpiryNotificationActionUseCase
import net.mullvad.mullvadvpn.usecase.NotificationAction
import net.mullvad.mullvadvpn.usecase.ScheduleNotificationAlarmUseCase
import org.koin.android.ext.android.getKoin
import org.koin.android.ext.koin.androidContext
import org.koin.core.context.loadKoinModules
import org.koin.core.context.startKoin

private const val LOG_TAG = "mullvad"

class ApplicationScope(private val cs: CoroutineScope) : CoroutineScope by cs

class MullvadApplication : Application() {

    override fun onCreate() {
        super.onCreate()
        Logger.setTag(LOG_TAG)
        if (!BuildConfig.DEBUG) {
            Logger.setMinSeverity(Severity.Info)
        }
        startKoin { androidContext(this@MullvadApplication) }
        loadKoinModules(listOf(appModule))
        with(getKoin()) {
            get<NotificationChannelFactory>()
            get<NotificationManager>()

            handleAccountExpiry(
                scope = get<ApplicationScope>(),
                accountExpiryUseCase = get<AccountExpiryNotificationActionUseCase>(),
                scheduleNotificationAlarmUseCase = get<ScheduleNotificationAlarmUseCase>(),
                accountExpiryNotificationProvider = get<AccountExpiryNotificationProvider>(),
            )
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
                        scheduleNotificationAlarmUseCase(
                            context = this@MullvadApplication,
                            accountExpiry = null,
                        )
                    }

                    is NotificationAction.ScheduleAlarm ->
                        scheduleNotificationAlarmUseCase(
                            context = this@MullvadApplication,
                            accountExpiry = action.alarmTime,
                        )
                }
            }
        }
    }
}
