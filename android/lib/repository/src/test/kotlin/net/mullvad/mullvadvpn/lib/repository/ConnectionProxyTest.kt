package net.mullvad.mullvadvpn.lib.repository

import android.content.Intent
import arrow.core.left
import arrow.core.right
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.PrepareError
import net.mullvad.mullvadvpn.lib.model.Prepared
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.Test

class ConnectionProxyTest {

    private val mockManagementService: ManagementService = mockk(relaxed = true)
    private val mockVpnPermissionRepository: PrepareVpnUseCase = mockk()
    private val mockTranslationRepository: RelayLocationTranslationRepository =
        mockk(relaxed = true)

    private val connectionProxy: ConnectionProxy =
        ConnectionProxy(
            managementService = mockManagementService,
            prepareVpnUseCase = mockVpnPermissionRepository,
            translationRepository = mockTranslationRepository,
        )

    @Test
    fun `connect with vpn permission allowed should call managementService connect`() = runTest {
        every { mockVpnPermissionRepository.invoke() } returns Prepared.right()
        connectionProxy.connect()
        coVerify(exactly = 1) { mockManagementService.connect() }
    }

    @Test
    fun `connect with vpn permission not allowed should not call managementService connect`() =
        runTest {
            every { mockVpnPermissionRepository.invoke() } returns
                PrepareError.NotPrepared(Intent()).left()
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
