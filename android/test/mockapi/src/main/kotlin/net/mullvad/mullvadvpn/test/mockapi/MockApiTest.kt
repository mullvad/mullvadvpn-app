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
import net.mullvad.mullvadvpn.test.mockapi.server.MockApiRouter
import net.mullvad.mullvadvpn.test.mockapi.server.MockServer
import net.mullvad.mullvadvpn.test.mockapi.server.port
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.extension.RegisterExtension

abstract class MockApiTest {

    @RegisterExtension @JvmField val rule = CaptureScreenshotOnFailedTestRule(LOG_TAG)

    @RegisterExtension
    @JvmField
    val permissionRule: GrantPermissionExtension =
        GrantPermissionExtension.grant(WRITE_EXTERNAL_STORAGE, READ_EXTERNAL_STORAGE)

    protected val apiRouter = MockApiRouter()
    private val mockApiServer = MockServer.createWithRouter(apiRouter)

    lateinit var device: UiDevice
    lateinit var targetContext: Context
    lateinit var app: AppInteractor
    lateinit var endpoint: ApiEndpointOverride

    @BeforeEach
    open fun setup() {
        Logger.setTag(LOG_TAG)

        device = UiDevice.getInstance(InstrumentationRegistry.getInstrumentation())
        targetContext = InstrumentationRegistry.getInstrumentation().targetContext

        mockApiServer.start()
        Logger.d("Mocked web server started using port: ${mockApiServer.port()}")
        endpoint = createEndpoint(mockApiServer.port())

        Logger.d("targetContext packageName: ${targetContext.packageName}")
        app = AppInteractor(device, targetContext, endpoint)
    }

    @AfterEach
    open fun teardown() {
        mockApiServer.stop()
    }

    private fun createEndpoint(port: Int): ApiEndpointOverride {
        return ApiEndpointOverride(
            InetAddress.getLocalHost().hostName,
            InetAddress.getLocalHost().hostAddress!!,
            port,
            disableTls = true,
            sigsumTrustedPubkeys = "",
        )
    }
}
