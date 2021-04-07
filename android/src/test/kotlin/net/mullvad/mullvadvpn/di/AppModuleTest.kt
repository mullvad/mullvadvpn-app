package net.mullvad.mullvadvpn.di

import android.os.Messenger
import io.mockk.mockk
import io.mockk.unmockkAll
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.MessageHandler
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnection
import net.mullvad.mullvadvpn.ui.serviceconnection.SplitTunneling
import org.junit.After
import org.junit.Rule
import org.junit.Test
import org.koin.core.parameter.parametersOf
import org.koin.core.qualifier.named
import org.koin.core.scope.Scope
import org.koin.test.KoinTest
import org.koin.test.KoinTestRule
import kotlin.test.assertEquals

class AppModuleTest : KoinTest {

    @get:Rule
    val koinTestRule = KoinTestRule.create {
        modules(appModule)
    }

    @After
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun test_scope_linking() {
        val appsScope: Scope = getKoin().createScope(APPS_SCOPE, named(APPS_SCOPE))
        val serviceConnectionScope = getKoin().createScope<ServiceConnection>()

        appsScope.linkTo(serviceConnectionScope)

        val mockedMessenger = mockk<Messenger>()
        val mockedEventMessageHandler = mockk<MessageHandler<Event>>(relaxed = true)
        val serviceConnectionSplitTunneling = serviceConnectionScope.get<SplitTunneling>(
            parameters = { parametersOf(mockedMessenger, mockedEventMessageHandler) }
        )

        assertEquals(appsScope.get<SplitTunneling>(), serviceConnectionSplitTunneling)
    }
}
