package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.DeadObjectException
import android.os.Looper
import android.os.Messenger
import android.util.Log
import io.mockk.MockKAnnotations
import io.mockk.Runs
import io.mockk.every
import io.mockk.impl.annotations.MockK
import io.mockk.just
import io.mockk.mockk
import io.mockk.mockkObject
import io.mockk.mockkStatic
import io.mockk.slot
import io.mockk.unmockkAll
import kotlin.reflect.KClass
import kotlin.test.assertEquals
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.EventDispatcher
import net.mullvad.mullvadvpn.lib.ipc.Request
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class ConnectionProxyTest {

    @MockK private lateinit var mockedMainLooper: Looper

    @MockK private lateinit var connection: Messenger

    @MockK private lateinit var mockedDispatchingHandler: EventDispatcher
    lateinit var connectionProxy: ConnectionProxy

    @BeforeEach
    fun setup() {
        mockkStatic(Looper::class)
        mockkStatic(Log::class)
        MockKAnnotations.init(this)
        mockkObject(Request.Connect, Request.Disconnect)
        every { Request.Connect.message } returns mockk()
        every { Request.Disconnect.message } returns mockk()
        every { Looper.getMainLooper() } returns mockedMainLooper
        every { Log.e(any(), any()) } returns mockk(relaxed = true)
    }

    @AfterEach
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun `initialize connection proxy should work`() {
        // Arrange
        val eventType = slot<KClass<Event.TunnelStateChange>>()
        every { mockedDispatchingHandler.registerHandler(capture(eventType), any()) } just Runs
        // Create ConnectionProxy instance and assert initial Event type
        connectionProxy = ConnectionProxy(connection, mockedDispatchingHandler)
        assertEquals(Event.TunnelStateChange::class, eventType.captured.java.kotlin)
    }

    @Test
    fun `normal connect and disconnect should not crash`() {
        // Arrange
        every { connection.send(any()) } just Runs
        every { mockedDispatchingHandler.registerHandler(any<KClass<Event>>(), any()) } just Runs
        // Act and Assert no crashes
        connectionProxy = ConnectionProxy(connection, mockedDispatchingHandler)
        connectionProxy.connect()
        connectionProxy.disconnect()
    }

    @Test
    fun `connect should catch DeadObjectException`() {
        // Arrange
        every { connection.send(any()) } throws DeadObjectException()
        every { mockedDispatchingHandler.registerHandler(any<KClass<Event>>(), any()) } just Runs
        // Act and Assert no crashes
        connectionProxy = ConnectionProxy(connection, mockedDispatchingHandler)
        connectionProxy.connect()
    }
}
