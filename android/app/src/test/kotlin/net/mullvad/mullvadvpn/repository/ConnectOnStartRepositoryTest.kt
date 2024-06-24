package net.mullvad.mullvadvpn.repository

import android.content.ComponentName
import android.content.pm.PackageManager
import app.cash.turbine.test
import io.mockk.Runs
import io.mockk.every
import io.mockk.just
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.test.runTest
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class ConnectOnStartRepositoryTest {

    private val mockPackageManager: PackageManager = mockk()
    private val mockComponentName: ComponentName = mockk()

    private lateinit var connectOnStartRepository: ConnectOnStartRepository

    @BeforeEach
    fun setUp() {
        every { mockPackageManager.getComponentEnabledSetting(mockComponentName) } returns
            PackageManager.COMPONENT_ENABLED_STATE_DEFAULT

        connectOnStartRepository =
            ConnectOnStartRepository(
                packageManager = mockPackageManager,
                bootCompletedComponentName = mockComponentName
            )
    }

    @Test
    fun `connectOnStart should emit false when default state is returned by package manager`() =
        runTest {
            // Assert
            connectOnStartRepository.connectOnStart.test { assertEquals(false, awaitItem()) }
        }

    @Test
    fun `when setting connectOnStart true should call package manager and update connectOnStart`() =
        runTest {
            // Arrange
            val targetState = true
            every {
                mockPackageManager.setComponentEnabledSetting(
                    mockComponentName,
                    PackageManager.COMPONENT_ENABLED_STATE_ENABLED,
                    PackageManager.DONT_KILL_APP
                )
            } just Runs
            every { mockPackageManager.getComponentEnabledSetting(mockComponentName) } returns
                PackageManager.COMPONENT_ENABLED_STATE_ENABLED

            // Act, Assert
            connectOnStartRepository.connectOnStart.test {
                assertEquals(false, awaitItem()) // Default state
                connectOnStartRepository.setConnectOnStart(targetState)
                verify {
                    mockPackageManager.setComponentEnabledSetting(
                        mockComponentName,
                        PackageManager.COMPONENT_ENABLED_STATE_ENABLED,
                        PackageManager.DONT_KILL_APP
                    )
                }
                assertEquals(targetState, awaitItem())
            }
        }
}
