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

class AutoStartAndConnectOnBootRepositoryTest {

    private val mockPackageManager: PackageManager = mockk()
    private val mockComponentName: ComponentName = mockk()

    private lateinit var autoStartAndConnectOnBootRepository: AutoStartAndConnectOnBootRepository

    @BeforeEach
    fun setUp() {
        every { mockPackageManager.getComponentEnabledSetting(mockComponentName) } returns
            PackageManager.COMPONENT_ENABLED_STATE_DEFAULT

        autoStartAndConnectOnBootRepository =
            AutoStartAndConnectOnBootRepository(
                packageManager = mockPackageManager,
                bootCompletedComponentName = mockComponentName,
            )
    }

    @Test
    fun `autoStartAndConnectOnBoot should emit false when default state is returned by package manager`() =
        runTest {
            // Assert
            autoStartAndConnectOnBootRepository.autoStartAndConnectOnBoot.test {
                assertEquals(false, awaitItem())
            }
        }

    @Test
    fun `when setting autoStartAndConnectOnBoot to true should call package manager and update autoStartAndConnectOnBoot`() =
        runTest {
            // Arrange
            val targetState = true
            every {
                mockPackageManager.setComponentEnabledSetting(
                    mockComponentName,
                    PackageManager.COMPONENT_ENABLED_STATE_ENABLED,
                    PackageManager.DONT_KILL_APP,
                )
            } just Runs
            every { mockPackageManager.getComponentEnabledSetting(mockComponentName) } returns
                PackageManager.COMPONENT_ENABLED_STATE_ENABLED

            // Act, Assert
            autoStartAndConnectOnBootRepository.autoStartAndConnectOnBoot.test {
                assertEquals(false, awaitItem()) // Default state
                autoStartAndConnectOnBootRepository.setAutoStartAndConnectOnBoot(targetState)
                verify {
                    mockPackageManager.setComponentEnabledSetting(
                        mockComponentName,
                        PackageManager.COMPONENT_ENABLED_STATE_ENABLED,
                        PackageManager.DONT_KILL_APP,
                    )
                }
                assertEquals(targetState, awaitItem())
            }
        }
}
