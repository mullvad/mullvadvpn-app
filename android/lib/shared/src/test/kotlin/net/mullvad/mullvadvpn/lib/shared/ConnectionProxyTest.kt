package net.mullvad.mullvadvpn.lib.shared

import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.Test

class ConnectionProxyTest {

    private val mockManagementService: ManagementService = mockk(relaxed = true)
    private val mockVpnPermissionRepository: VpnProfileUseCase = mockk()
    private val mockTranslationRepository: RelayLocationTranslationRepository =
        mockk(relaxed = true)

    private val connectionProxy: ConnectionProxy =
        ConnectionProxy(
            managementService = mockManagementService,
            vpnProfileUseCase = mockVpnPermissionRepository,
            translationRepository = mockTranslationRepository,
        )

    @Test
    fun `connect with vpn permission allowed should call managementService connect`() = runTest {
        every { mockVpnPermissionRepository.prepareVpn() } returns true
        connectionProxy.connect()
        coVerify(exactly = 1) { mockManagementService.connect() }
    }

    @Test
    fun `connect with vpn permission not allowed should not call managementService connect`() =
        runTest {
            every { mockVpnPermissionRepository.prepareVpn() } returns false
            connectionProxy.connect()
            coVerify(exactly = 0) { mockManagementService.connect() }
        }

    @Test
    fun `disconnect should call managementService disconnect`() = runTest {
        connectionProxy.disconnect()
        coVerify(exactly = 1) { mockManagementService.disconnect() }
    }

    @Test
    fun `reconnect should call managementService reconnect`() = runTest {
        connectionProxy.reconnect()
        coVerify(exactly = 1) { mockManagementService.reconnect() }
    }

    @AfterEach
    fun tearDown() {
        unmockkAll()
    }
}
