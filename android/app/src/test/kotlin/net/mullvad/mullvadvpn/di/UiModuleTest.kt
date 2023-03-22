package net.mullvad.mullvadvpn.di

import android.os.Messenger
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.MessageDispatcher
import net.mullvad.mullvadvpn.ui.serviceconnection.SplitTunneling
import org.junit.After
import org.junit.Rule
import org.junit.Test
import org.koin.core.parameter.parametersOf
import org.koin.core.qualifier.named
import org.koin.core.scope.Scope
import org.koin.test.KoinTest
import org.koin.test.KoinTestRule

class UiModuleTest : KoinTest {

    @get:Rule val koinTestRule = KoinTestRule.create { modules(uiModule) }

    @After
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun test_scope_linking() {
        val appsScope: Scope = getKoin().createScope(APPS_SCOPE, named(APPS_SCOPE))
        val serviceConnectionScope =
            getKoin().createScope(SERVICE_CONNECTION_SCOPE, named(SERVICE_CONNECTION_SCOPE))

        appsScope.linkTo(serviceConnectionScope)

        val mockedMessenger = mockk<Messenger>()
        val mockedEventMessageHandler = mockk<MessageDispatcher<Event>>(relaxed = true)
        val serviceConnectionSplitTunneling =
            serviceConnectionScope.get<SplitTunneling>(
                parameters = { parametersOf(mockedMessenger, mockedEventMessageHandler) }
            )

        assertEquals(appsScope.get<SplitTunneling>(), serviceConnectionSplitTunneling)
    }
}
