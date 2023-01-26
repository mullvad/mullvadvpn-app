package net.mullvad.mullvadvpn.test.mockapi

import android.Manifest.permission.READ_EXTERNAL_STORAGE
import android.Manifest.permission.WRITE_EXTERNAL_STORAGE
import android.content.Context
import android.util.Log
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.rule.GrantPermissionRule
import androidx.test.runner.AndroidJUnit4
import androidx.test.uiautomator.UiDevice
import java.net.InetAddress
import java.net.InetSocketAddress
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpoint
import net.mullvad.mullvadvpn.lib.endpoint.CustomApiEndpointConfiguration
import net.mullvad.mullvadvpn.test.common.interactor.AppInteractor
import net.mullvad.mullvadvpn.test.common.rule.CaptureScreenshotOnFailedTestRule
import net.mullvad.mullvadvpn.test.mockapi.constant.LOG_TAG
import net.mullvad.mullvadvpn.test.mockapi.constant.MOCK_SERVER_LOCALHOST_ADDRESS
import okhttp3.mockwebserver.MockWebServer
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
abstract class MockApiTest {

    @Rule
    @JvmField
    val rule = CaptureScreenshotOnFailedTestRule(LOG_TAG)

    @Rule
    @JvmField
    val permissionRule: GrantPermissionRule = GrantPermissionRule.grant(
        WRITE_EXTERNAL_STORAGE,
        READ_EXTERNAL_STORAGE
    )

    protected val apiDispatcher = MockApiDispatcher()
    private val mockWebServer = MockWebServer().apply {
        dispatcher = apiDispatcher
    }

    lateinit var device: UiDevice
    lateinit var targetContext: Context
    lateinit var app: AppInteractor
    lateinit var endpoint: CustomApiEndpointConfiguration

    @Before
    open fun setup() {
        device = UiDevice.getInstance(InstrumentationRegistry.getInstrumentation())
        targetContext = InstrumentationRegistry.getInstrumentation().targetContext

        app = AppInteractor(
            device,
            targetContext
        )

        mockWebServer.start()
        Log.d(LOG_TAG, "Mocked web server started using port: ${mockWebServer.port}")
        endpoint = createEndpoint(mockWebServer.port)
    }

    @After
    open fun teardown() {
        mockWebServer.shutdown()
    }

    private fun createEndpoint(port: Int): CustomApiEndpointConfiguration {
        val mockApiSocket = InetSocketAddress(
            InetAddress.getByName(MOCK_SERVER_LOCALHOST_ADDRESS),
            port
        )
        val api = ApiEndpoint(
            address = mockApiSocket,
            disableAddressCache = true,
            disableTls = true,
            forceDirectConnection = true
        )
        return CustomApiEndpointConfiguration(api)
    }
}
