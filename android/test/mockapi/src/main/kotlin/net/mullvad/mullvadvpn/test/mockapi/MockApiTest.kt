package net.mullvad.mullvadvpn.test.mockapi

import android.Manifest.permission.READ_EXTERNAL_STORAGE
import android.Manifest.permission.WRITE_EXTERNAL_STORAGE
import android.content.Context
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.UiDevice
import co.touchlab.kermit.Logger
import de.mannodermaus.junit5.extensions.GrantPermissionExtension
import java.net.InetAddress
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointOverride
import net.mullvad.mullvadvpn.test.common.interactor.AppInteractor
import net.mullvad.mullvadvpn.test.common.rule.CaptureScreenshotOnFailedTestRule
import net.mullvad.mullvadvpn.test.mockapi.constant.LOG_TAG
import net.mullvad.mullvadvpn.test.mockapi.constant.PACKAGE_NAME
import okhttp3.mockwebserver.MockWebServer
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.extension.RegisterExtension

abstract class MockApiTest {

    @RegisterExtension @JvmField val rule = CaptureScreenshotOnFailedTestRule(LOG_TAG)

    @RegisterExtension
    @JvmField
    val permissionRule: GrantPermissionExtension =
        GrantPermissionExtension.grant(WRITE_EXTERNAL_STORAGE, READ_EXTERNAL_STORAGE)

    protected val apiDispatcher = MockApiDispatcher()
    private val mockWebServer = MockWebServer().apply { dispatcher = apiDispatcher }

    lateinit var device: UiDevice
    lateinit var targetContext: Context
    lateinit var app: AppInteractor
    lateinit var endpoint: ApiEndpointOverride

    @BeforeEach
    open fun setup() {
        Logger.setTag(LOG_TAG)

        device = UiDevice.getInstance(InstrumentationRegistry.getInstrumentation())
        targetContext = InstrumentationRegistry.getInstrumentation().targetContext

        app = AppInteractor(device, targetContext, PACKAGE_NAME)

        mockWebServer.start()
        Logger.d("Mocked web server started using port: ${mockWebServer.port}")
        endpoint = createEndpoint(mockWebServer.port)
    }

    @AfterEach
    open fun teardown() {
        mockWebServer.shutdown()
    }

    private fun createEndpoint(port: Int): ApiEndpointOverride {
        return ApiEndpointOverride(
            InetAddress.getLocalHost().hostName,
            port,
            disableTls = true,
        )
    }
}
