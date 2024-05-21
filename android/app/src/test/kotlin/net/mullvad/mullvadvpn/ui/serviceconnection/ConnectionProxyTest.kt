package net.mullvad.mullvadvpn.ui.serviceconnection

import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.permission.VpnPermissionRepository
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.Test

class ConnectionProxyTest {

    private val mockManagementService: ManagementService = mockk(relaxed = true)
    private val mockVpnPermissionRepository: VpnPermissionRepository = mockk()

    private val connectionProxy: ConnectionProxy =
        ConnectionProxy(
            managementService = mockManagementService,
            vpnPermissionRepository = mockVpnPermissionRepository
        )

    @Test
    fun `connect with vpn permission allowed should call managementService connect`() = runTest {
        every { mockVpnPermissionRepository.hasVpnPermission() } returns true
        connectionProxy.connect()
        coVerify(exactly = 1) { mockManagementService.connect() }
    }

    @Test
    fun `connect with vpn permission not allowed should not call managementService connect`() =
        runTest {
            every { mockVpnPermissionRepository.hasVpnPermission() } returns false
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
