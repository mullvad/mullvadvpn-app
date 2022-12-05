package net.mullvad.mullvadvpn.service

import android.os.DeadObjectException
import android.os.Looper
import android.os.Messenger
import io.mockk.mockkStatic
import io.mockk.MockKAnnotations
import io.mockk.mockkObject
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import io.mockk.just
import io.mockk.Runs
import io.mockk.impl.annotations.MockK
import kotlin.reflect.KClass
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.EventDispatcher
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionDeviceDataSource
import net.mullvad.mullvadvpn.util.JobTracker
import org.junit.After
import org.junit.Before
import org.junit.Test

class ServiceConnectionDeviceDataSourceTest {
    private val tracker = JobTracker()

    @MockK
    private lateinit var mockedMainLooper: Looper

    @MockK
    private lateinit var mockedDispatchingHandler: EventDispatcher

    @MockK
    private lateinit var connection: Messenger

    lateinit var serviceConnectionDeviceDataSource: ServiceConnectionDeviceDataSource

    @Before
    fun setup() {
        mockkStatic(Looper::class)
        MockKAnnotations.init(this)
        mockkObject(Request.GetDevice, Request.RefreshDeviceState)
        every { Request.GetDevice.message } returns mockk()
        every { Request.RefreshDeviceState.message } returns mockk()
        every { Looper.getMainLooper() } returns mockedMainLooper
    }

    @After
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun test_get_devices_list() {
        // Arrange
        every { connection.send(any()) } just Runs
        every {
            mockedDispatchingHandler.registerHandler(any<KClass<Event>>(), any())
        } just Runs
        serviceConnectionDeviceDataSource =
            ServiceConnectionDeviceDataSource(connection, mockedDispatchingHandler)
        serviceConnectionDeviceDataSource.getDevice()
    }

    @Test
    fun test_catch_exception_on_devices_list() {
        // Arrange
        every { connection.send(any()) } throws DeadObjectException()
        every {
            mockedDispatchingHandler.registerHandler(any<KClass<Event>>(), any())
        } just Runs
        serviceConnectionDeviceDataSource =
            ServiceConnectionDeviceDataSource(connection, mockedDispatchingHandler)
        serviceConnectionDeviceDataSource.getDevice()
    }
}
