package net.mullvad.mullvadvpn.usecase

import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.ErrorState
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause.NoRelaysMatchSelectedPort
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause.TunnelParameterError
import net.mullvad.mullvadvpn.lib.model.InAppNotification
import net.mullvad.mullvadvpn.lib.model.ParameterGenerationError
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.repository.ConnectionProxy
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.usecase.inappnotification.TunnelStateNotificationUseCase
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertNull
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class TunnelStateNotificationUseCaseTest {

    private val mockConnectionProxy: ConnectionProxy = mockk()
    private val mockRelayListRepository: RelayListRepository = mockk()
    private val mockSettingsRepository: SettingsRepository = mockk()

    private lateinit var tunnelStateNotificationUseCase: TunnelStateNotificationUseCase

    private val tunnelState = MutableStateFlow<TunnelState>(TunnelState.Disconnected())
    private val portRanges = MutableStateFlow<List<PortRange>>(emptyList())
    private val settingsFlow = MutableStateFlow<Settings>(mockk(relaxed = true))

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
        every { mockConnectionProxy.tunnelState } returns tunnelState
        every { mockRelayListRepository.portRanges } returns portRanges
        every { mockSettingsRepository.settingsUpdates } returns settingsFlow

        tunnelStateNotificationUseCase =
            TunnelStateNotificationUseCase(
                connectionProxy = mockConnectionProxy,
                relayListRepository = mockRelayListRepository,
                settingsRepository = mockSettingsRepository,
            )
    }

    @AfterEach
    fun teardown() {
        unmockkAll()
    }

    @Test
    fun `initial state should be empty`() = runTest {
        // Arrange, Act, Assert
        tunnelStateNotificationUseCase().test { assertNull(awaitItem()) }
    }

    @Test
    fun `when TunnelState is error use case should emit TunnelStateError notification`() = runTest {
        tunnelStateNotificationUseCase().test {
            // Arrange, Act
            assertNull(awaitItem())
            val errorState: ErrorState = mockk()
            every { errorState.cause } returns mockk()
            tunnelState.emit(TunnelState.Error(errorState))

            // Assert
            assertEquals(InAppNotification.TunnelStateError(errorState), awaitItem())
        }
    }

    @Test
    fun `when TunnelState is Disconnecting with blocking use case should emit TunnelStateBlocked notification`() =
        runTest {
            tunnelStateNotificationUseCase().test {
                // Arrange, Act
                assertNull(awaitItem())
                tunnelState.emit(TunnelState.Disconnecting(ActionAfterDisconnect.Block))

                // Assert
                assertEquals(InAppNotification.TunnelStateBlocked, awaitItem())
            }
        }

    @Test
    fun `when error cause is TunnelParameterError and port is not in range use case should emit NoRelaysMatchSelectedPort error`() =
        runTest {
            tunnelStateNotificationUseCase().test {
                // Arrange, Act
                assertNull(awaitItem())
                val errorState: ErrorState = mockk()
                every { errorState.isBlocking } returns true
                every { errorState.cause } returns
                    TunnelParameterError(ParameterGenerationError.NoMatchingRelay)
                val settings: Settings = mockk()
                every { settings.obfuscationSettings.wireguardPort } returns
                    Constraint.Only(Port(1))
                val portRange = PortRange(2..3)
                settingsFlow.emit(settings)
                portRanges.emit(listOf(portRange))
                tunnelState.emit(TunnelState.Error(errorState))

                // Assert
                val item = awaitItem()
                assertTrue {
                    (item as InAppNotification.TunnelStateError).error.cause is
                        NoRelaysMatchSelectedPort
                }
            }
        }

    @Test
    fun `when error cause is TunnelParameterError and port is in range use case should emit TunnelParameterError error`() =
        runTest {
            tunnelStateNotificationUseCase().test {
                // Arrange, Act
                assertNull(awaitItem())
                val errorState: ErrorState = mockk()
                every { errorState.isBlocking } returns true
                every { errorState.cause } returns
                    TunnelParameterError(ParameterGenerationError.NoMatchingRelay)
                val settings: Settings = mockk()
                every { settings.obfuscationSettings.wireguardPort } returns
                    Constraint.Only(Port(2))
                val portRange = PortRange(2..3)
                settingsFlow.emit(settings)
                portRanges.emit(listOf(portRange))
                tunnelState.emit(TunnelState.Error(errorState))

                // Assert
                val item = awaitItem()
                assertEquals(InAppNotification.TunnelStateError(errorState), item)
                assertTrue {
                    (item as InAppNotification.TunnelStateError).error.cause is TunnelParameterError
                }
            }
        }
}
