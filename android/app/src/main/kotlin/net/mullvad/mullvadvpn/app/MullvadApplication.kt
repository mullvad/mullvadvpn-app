package net.mullvad.mullvadvpn.app

import android.app.Application
import android.content.Context
import androidx.compose.runtime.Composer
import androidx.compose.runtime.ExperimentalComposeRuntimeApi
import androidx.compose.runtime.tooling.ComposeStackTraceMode
import androidx.lifecycle.ViewModel
import co.touchlab.kermit.Logger
import co.touchlab.kermit.Severity
import dev.zacsweers.metro.AppScope
import dev.zacsweers.metro.ContributesBinding
import dev.zacsweers.metro.DependencyGraph
import dev.zacsweers.metro.Includes
import dev.zacsweers.metro.Inject
import dev.zacsweers.metro.Provider
import dev.zacsweers.metro.SingleIn
import dev.zacsweers.metro.createGraph
import dev.zacsweers.metrox.android.MetroAppComponentProviders
import dev.zacsweers.metrox.android.MetroApplication
import dev.zacsweers.metrox.viewmodel.ManualViewModelAssistedFactory
import dev.zacsweers.metrox.viewmodel.MetroViewModelFactory
import dev.zacsweers.metrox.viewmodel.ViewModelAssistedFactory
import dev.zacsweers.metrox.viewmodel.ViewModelGraph
import java.io.IOException
import kotlin.reflect.KClass
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.app.util.FileLogWriter
import net.mullvad.mullvadvpn.di.AppModuleBindings
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

@DependencyGraph(
    bindingContainers = [AppModuleBindings::class],
    scope = AppScope::class
)
interface AppGraph : MetroAppComponentProviders, ViewModelGraph {
    //    val splashCompleteRepository: SplashCompleteRepository

    val context: Context

    @DependencyGraph.Factory
    interface Factory {
        fun create(@Includes appModuleBindings: AppModuleBindings): AppGraph
    }
}

@Inject
@ContributesBinding(AppScope::class)
@SingleIn(AppScope::class)
class MyViewModelFactory(
    override val viewModelProviders: Map<KClass<out ViewModel>, Provider<ViewModel>>,
    override val assistedFactoryProviders:
        Map<KClass<out ViewModel>, Provider<ViewModelAssistedFactory>>,
    override val manualAssistedFactoryProviders:
        Map<KClass<out ManualViewModelAssistedFactory>, Provider<ManualViewModelAssistedFactory>>,
) : MetroViewModelFactory()

@OptIn(ExperimentalComposeRuntimeApi::class)
class MullvadApplication : Application(), MetroApplication {
    private val appGraph by lazy { createGraph<AppGraph>() }
    override val appComponentProviders: MetroAppComponentProviders
        get() = appGraph

    override fun onCreate() {
        super.onCreate()
        Logger.setTag(LOG_TAG)
        if (!BuildConfig.DEBUG) {
            Logger.setMinSeverity(Severity.Info)
        }
        // Improve compose stack traces
        // Comes with a performance penalty, so only enable in debug builds
        if (BuildConfig.DEBUG) {
            Composer.setDiagnosticStackTraceMode(ComposeStackTraceMode.SourceInformation)
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
        try {
            val fileLogWriter =
                FileLogWriter(
                    logDir = this.filesDir.toPath().resolve(KERMIT_FILE_LOG_DIR_NAME),
                    scope = scope,
                )
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
