package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.DeadObjectException
import android.os.Looper
import android.os.Messenger
import io.mockk.MockKAnnotations
import io.mockk.Runs
import io.mockk.every
import io.mockk.impl.annotations.MockK
import io.mockk.just
import io.mockk.mockk
import io.mockk.mockkObject
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import kotlin.reflect.KClass
import net.mullvad.mullvadvpn.lib.common.util.JobTracker
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.EventDispatcher
import net.mullvad.mullvadvpn.lib.ipc.Request
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class ServiceConnectionDeviceDataSourceTest {
    private val tracker = JobTracker()

    @MockK private lateinit var mockedMainLooper: Looper

    @MockK private lateinit var mockedDispatchingHandler: EventDispatcher

    @MockK private lateinit var connection: Messenger

    lateinit var serviceConnectionDeviceDataSource: ServiceConnectionDeviceDataSource

    @BeforeEach
    fun setup() {
        mockkStatic(Looper::class)
        mockkStatic(android.util.Log::class)
        MockKAnnotations.init(this)
        mockkObject(Request.GetDevice, Request.RefreshDeviceState)
        every { Request.GetDevice.message } returns mockk()
        every { Request.RefreshDeviceState.message } returns mockk()
        every { Looper.getMainLooper() } returns mockedMainLooper
        every { android.util.Log.e(any(), any()) } returns mockk(relaxed = true)
    }

    @AfterEach
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun test_get_devices_list() {
        // Arrange
        every { connection.send(any()) } just Runs
        every { mockedDispatchingHandler.registerHandler(any<KClass<Event>>(), any()) } just Runs
        // Act and Assert no crashes
        serviceConnectionDeviceDataSource =
            ServiceConnectionDeviceDataSource(connection, mockedDispatchingHandler)
        serviceConnectionDeviceDataSource.getDevice()
    }

    @Test
    fun test_catch_exception_on_devices_list() {
        // Arrange
        every { connection.send(any()) } throws DeadObjectException()
        every { mockedDispatchingHandler.registerHandler(any<KClass<Event>>(), any()) } just Runs
        // Act and Assert no crashes
        serviceConnectionDeviceDataSource =
            ServiceConnectionDeviceDataSource(connection, mockedDispatchingHandler)
        serviceConnectionDeviceDataSource.getDevice()
    }
}
