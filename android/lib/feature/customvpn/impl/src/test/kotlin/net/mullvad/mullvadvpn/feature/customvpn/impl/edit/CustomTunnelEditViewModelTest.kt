package net.mullvad.mullvadvpn.feature.customvpn.impl.edit

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import arrow.core.left
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import java.net.InetSocketAddress
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.feature.customvpn.impl.VALID_ALLOWED_IP
import net.mullvad.mullvadvpn.feature.customvpn.impl.VALID_ENDPOINT_HOST
import net.mullvad.mullvadvpn.feature.customvpn.impl.VALID_ENDPOINT_PORT
import net.mullvad.mullvadvpn.feature.customvpn.impl.VALID_NAME
import net.mullvad.mullvadvpn.feature.customvpn.impl.VALID_PRIVATE_KEY
import net.mullvad.mullvadvpn.feature.customvpn.impl.VALID_PUBLIC_KEY
import net.mullvad.mullvadvpn.feature.customvpn.impl.VALID_TUNNEL_IP
import net.mullvad.mullvadvpn.feature.customvpn.impl.createSampleCustomVpnConfig
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.CustomVpnConfig
import net.mullvad.mullvadvpn.lib.model.KeyParseError
import net.mullvad.mullvadvpn.lib.model.ParsePortError
import net.mullvad.mullvadvpn.lib.model.PeerConfig
import net.mullvad.mullvadvpn.lib.model.SetCustomVpnConfigError
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.model.TunnelConfig
import net.mullvad.mullvadvpn.lib.model.WireguardKey
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class CustomTunnelEditViewModelTest {

    private val mockSettingsRepository: SettingsRepository = mockk()
    private val mockManagementService: ManagementService = mockk()

    private val settingsUpdates = MutableStateFlow<Settings?>(null)

    private lateinit var viewModel: CustomTunnelEditViewModel

    @BeforeEach
    fun setup() {
        every { mockSettingsRepository.settingsUpdates } returns settingsUpdates

        viewModel =
            CustomTunnelEditViewModel(
                settingsRepository = mockSettingsRepository,
                managementService = mockManagementService,
            )
    }

    @AfterEach
    fun tearDown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun `initial state should be loading`() = runTest {
        viewModel.uiState.test {
            val initial = awaitItem()
            assertIs<Lc.Loading<Boolean>>(initial)
            assertFalse(initial.value)
        }
    }

    @Test
    fun `when settings updates emit null custom vpn config uiState should emit default empty form data`() =
        runTest {
            // Act, Assert
            viewModel.uiState.test {
                assertEquals(Lc.Loading(false), awaitItem())

                initSettings(customVpnConfig = null)

                val content = awaitItem()
                assertIs<Lc.Content<CustomTunnelEditUiState>>(content)
                assertFormDataMatches(PersonalVpnFormData(), content.value.formData)
                assertNull(content.value.formErrors)
            }
        }

    @Test
    fun `when settings updates emit custom vpn config uiState should emit populated form data`() =
        runTest {
            // Arrange
            val customVpnConfig = createSampleCustomVpnConfig()

            // Act, Assert
            viewModel.uiState.test {
                assertEquals(Lc.Loading(false), awaitItem())

                initSettings(customVpnConfig = customVpnConfig)

                val content = awaitItem()
                assertIs<Lc.Content<CustomTunnelEditUiState>>(content)
                assertFormDataMatches(
                    PersonalVpnFormData.from(customVpnConfig),
                    content.value.formData,
                )
                assertNull(content.value.formErrors)
            }
        }

    @Test
    fun `clearConfig should call management service clearPersonalVpn and reset formData`() =
        runTest {
            // Arrange
            val customVpnConfig = createSampleCustomVpnConfig()
            coEvery { mockManagementService.clearPersonalVpn() } returns Unit.right()

            // Act, Assert
            viewModel.uiState.test {
                assertEquals(Lc.Loading(false), awaitItem())

                initSettings(customVpnConfig = customVpnConfig)

                assertIs<Lc.Content<CustomTunnelEditUiState>>(awaitItem())

                viewModel.clearConfig()

                val updatedContent = awaitItem()
                assertIs<Lc.Content<CustomTunnelEditUiState>>(updatedContent)
                assertFormDataMatches(PersonalVpnFormData.from(null), updatedContent.value.formData)
            }

            coVerify(exactly = 1) { mockManagementService.clearPersonalVpn() }
        }

    @Test
    fun `when save with empty name should emit FormErrors side effect and set nameError`() =
        runTest {
            // Arrange
            initSettings()
            val invalidForm = createValidFormData().copy(name = "")

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.save(invalidForm)
                assertEquals(PersonalVpnSideEffect.FormErrors, awaitItem())
            }

            viewModel.uiState.test {
                val content = awaitItem()
                assertIs<Lc.Content<CustomTunnelEditUiState>>(content)
                assertEquals(FormDataError.EmptyName, content.value.formErrors?.nameError)
            }

            coVerify(exactly = 0) { mockManagementService.setPersonalVpnConfig(any()) }
        }

    @Test
    fun `when save with invalid private key should emit FormErrors side effect and set privateKeyError`() =
        runTest {
            // Arrange
            initSettings()
            val invalidPrivateKey = "invalid-key"
            val invalidForm = createValidFormData().copy(privateKey = invalidPrivateKey)

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.save(invalidForm)
                assertEquals(PersonalVpnSideEffect.FormErrors, awaitItem())
            }

            viewModel.uiState.test {
                val content = awaitItem()
                assertIs<Lc.Content<CustomTunnelEditUiState>>(content)
                val keyError = content.value.formErrors?.privateKeyError
                assertNotNull(keyError)
                assertEquals(invalidPrivateKey, keyError.errorValue)
                assertIs<KeyParseError.InvalidLength>(keyError.keyParseError)
            }

            coVerify(exactly = 0) { mockManagementService.setPersonalVpnConfig(any()) }
        }

    @Test
    fun `when save with invalid tunnel ip should emit FormErrors side effect and set tunnelIpErrors`() =
        runTest {
            // Arrange
            initSettings()
            val invalidIp = "999.999.999.999"
            val invalidForm = createValidFormData().copy(tunnelIps = setOf(invalidIp))

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.save(invalidForm)
                assertEquals(PersonalVpnSideEffect.FormErrors, awaitItem())
            }

            viewModel.uiState.test {
                val content = awaitItem()
                assertIs<Lc.Content<CustomTunnelEditUiState>>(content)
                val ipError = content.value.formErrors?.tunnelIpErrors?.get(invalidIp)
                assertEquals(FormDataError.TunnelIp(invalidIp), ipError)
            }

            coVerify(exactly = 0) { mockManagementService.setPersonalVpnConfig(any()) }
        }

    @Test
    fun `when save with invalid peer public key should emit FormErrors side effect and set peer publicKeyError`() =
        runTest {
            // Arrange
            initSettings()
            val invalidPublicKey = "invalid-peer-key"
            val peerId = "peer-test-id"
            val invalidPeer =
                createValidPeerFormData().copy(id = peerId, publicKey = invalidPublicKey)
            val invalidForm = createValidFormData().copy(peers = listOf(invalidPeer))

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.save(invalidForm)
                assertEquals(PersonalVpnSideEffect.FormErrors, awaitItem())
            }

            viewModel.uiState.test {
                val content = awaitItem()
                assertIs<Lc.Content<CustomTunnelEditUiState>>(content)
                val peerError = content.value.formErrors?.peerErrors?.get(peerId)
                assertNotNull(peerError)
                val keyError = peerError.publicKeyError
                assertNotNull(keyError)
                assertEquals(invalidPublicKey, keyError.errorValue)
                assertIs<KeyParseError.InvalidLength>(keyError.keyParseError)
            }

            coVerify(exactly = 0) { mockManagementService.setPersonalVpnConfig(any()) }
        }

    @Test
    fun `when save with invalid peer allowed ip should emit FormErrors side effect and set peer allowedIpErrors`() =
        runTest {
            // Arrange
            initSettings()
            val invalidAllowedIp = "invalid-cidr"
            val peerId = "peer-test-id"
            val invalidPeer =
                createValidPeerFormData().copy(id = peerId, allowedIps = setOf(invalidAllowedIp))
            val invalidForm = createValidFormData().copy(peers = listOf(invalidPeer))

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.save(invalidForm)
                assertEquals(PersonalVpnSideEffect.FormErrors, awaitItem())
            }

            viewModel.uiState.test {
                val content = awaitItem()
                assertIs<Lc.Content<CustomTunnelEditUiState>>(content)
                val peerError = content.value.formErrors?.peerErrors?.get(peerId)
                assertNotNull(peerError)
                assertEquals(
                    FormDataError.AllowedIp(invalidAllowedIp),
                    peerError.allowedIpErrors[invalidAllowedIp],
                )
            }

            coVerify(exactly = 0) { mockManagementService.setPersonalVpnConfig(any()) }
        }

    @Test
    fun `when save with invalid peer endpoint address should emit FormErrors side effect and set endpointAddressError`() =
        runTest {
            // Arrange
            initSettings()
            val invalidEndpoint = "not-an-ip-address"
            val peerId = "peer-test-id"
            val invalidPeer =
                createValidPeerFormData().copy(id = peerId, endpointAddress = invalidEndpoint)
            val invalidForm = createValidFormData().copy(peers = listOf(invalidPeer))

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.save(invalidForm)
                assertEquals(PersonalVpnSideEffect.FormErrors, awaitItem())
            }

            viewModel.uiState.test {
                val content = awaitItem()
                assertIs<Lc.Content<CustomTunnelEditUiState>>(content)
                val peerError = content.value.formErrors?.peerErrors?.get(peerId)
                assertNotNull(peerError)
                assertEquals(
                    FormDataError.EndpointAddress(invalidEndpoint),
                    peerError.endpointAddressError,
                )
            }

            coVerify(exactly = 0) { mockManagementService.setPersonalVpnConfig(any()) }
        }

    @Test
    fun `when save with non-numeric peer endpoint port should emit FormErrors side effect and set endpointPortError`() =
        runTest {
            // Arrange
            initSettings()
            val invalidPort = "not-a-number"
            val peerId = "peer-test-id"
            val invalidPeer =
                createValidPeerFormData().copy(id = peerId, endpointPort = invalidPort)
            val invalidForm = createValidFormData().copy(peers = listOf(invalidPeer))

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.save(invalidForm)
                assertEquals(PersonalVpnSideEffect.FormErrors, awaitItem())
            }

            viewModel.uiState.test {
                val content = awaitItem()
                assertIs<Lc.Content<CustomTunnelEditUiState>>(content)
                val peerError = content.value.formErrors?.peerErrors?.get(peerId)
                assertNotNull(peerError)
                val portError = peerError.endpointPortError
                assertNotNull(portError)
                assertEquals(invalidPort, portError.errorValue)
                assertIs<ParsePortError.NotANumber>(portError.parsePortError)
            }

            coVerify(exactly = 0) { mockManagementService.setPersonalVpnConfig(any()) }
        }

    @Test
    fun `when save with out-of-range peer endpoint port should emit FormErrors side effect and set endpointPortError`() =
        runTest {
            // Arrange
            initSettings()
            val outOfRangePort = "70000"
            val peerId = "peer-test-id"
            val invalidPeer =
                createValidPeerFormData().copy(id = peerId, endpointPort = outOfRangePort)
            val invalidForm = createValidFormData().copy(peers = listOf(invalidPeer))

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.save(invalidForm)
                assertEquals(PersonalVpnSideEffect.FormErrors, awaitItem())
            }

            viewModel.uiState.test {
                val content = awaitItem()
                assertIs<Lc.Content<CustomTunnelEditUiState>>(content)
                val peerError = content.value.formErrors?.peerErrors?.get(peerId)
                assertNotNull(peerError)
                val portError = peerError.endpointPortError
                assertNotNull(portError)
                assertEquals(outOfRangePort, portError.errorValue)
                assertIs<ParsePortError.OutOfRange>(portError.parsePortError)
            }

            coVerify(exactly = 0) { mockManagementService.setPersonalVpnConfig(any()) }
        }

    @Test
    fun `when save with valid form data succeeds should call management service and emit ConfigurationSaved`() =
        runTest {
            // Arrange
            initSettings()
            val validFormData = createValidFormData()
            val expectedPrivateKey = WireguardKey.from(VALID_PRIVATE_KEY).getOrNull()!!
            val expectedPublicKey = WireguardKey.from(VALID_PUBLIC_KEY).getOrNull()!!
            val expectedConfig =
                CustomVpnConfig(
                    tunnelConfig =
                        TunnelConfig(
                            privateKey = expectedPrivateKey,
                            tunnelIps = setOf(VALID_TUNNEL_IP),
                        ),
                    peerConfig =
                        listOf(
                            PeerConfig(
                                publicKey = expectedPublicKey,
                                endpoint =
                                    InetSocketAddress(VALID_ENDPOINT_HOST, VALID_ENDPOINT_PORT),
                                allowedIps = setOf(VALID_ALLOWED_IP),
                            )
                        ),
                    enabled = true,
                    name = VALID_NAME,
                )
            coEvery { mockManagementService.setPersonalVpnConfig(expectedConfig) } returns
                Unit.right()

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.save(validFormData)
                assertEquals(PersonalVpnSideEffect.ConfigurationSaved, awaitItem())
            }

            viewModel.uiState.test {
                val content = awaitItem()
                assertIs<Lc.Content<CustomTunnelEditUiState>>(content)
                assertFormDataMatches(
                    PersonalVpnFormData.from(expectedConfig),
                    content.value.formData,
                )
                assertNull(content.value.formErrors)
            }

            coVerify(exactly = 1) { mockManagementService.setPersonalVpnConfig(expectedConfig) }
        }

    @Test
    fun `when management service fails to set personal vpn config should emit FailedToSave side effect`() =
        runTest {
            // Arrange
            initSettings()
            val validFormData = createValidFormData()
            val saveError = SetCustomVpnConfigError.Unknown(RuntimeException("Save failed"))
            coEvery { mockManagementService.setPersonalVpnConfig(any()) } returns saveError.left()

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.save(validFormData)
                val sideEffect = awaitItem()
                assertIs<PersonalVpnSideEffect.FailedToSave>(sideEffect)
                assertEquals(saveError.toString(), sideEffect.reason)
            }

            coVerify(exactly = 1) { mockManagementService.setPersonalVpnConfig(any()) }
        }

    private fun initSettings(customVpnConfig: CustomVpnConfig? = null) {
        val mockSettings = mockk<Settings>(relaxed = true)
        every { mockSettings.customVpnConfig } returns customVpnConfig
        settingsUpdates.value = mockSettings
    }

    private fun assertFormDataMatches(
        expected: PersonalVpnFormData,
        actual: PersonalVpnFormData,
    ) {
        assertEquals(expected.enabled, actual.enabled)
        assertEquals(expected.name, actual.name)
        assertEquals(expected.privateKey, actual.privateKey)
        assertEquals(expected.tunnelIps, actual.tunnelIps)
        assertEquals(expected.peers.size, actual.peers.size)
        expected.peers.zip(actual.peers).forEach { (expectedPeer, actualPeer) ->
            assertEquals(expectedPeer.publicKey, actualPeer.publicKey)
            assertEquals(expectedPeer.allowedIps, actualPeer.allowedIps)
            assertEquals(expectedPeer.endpointAddress, actualPeer.endpointAddress)
            assertEquals(expectedPeer.endpointPort, actualPeer.endpointPort)
        }
    }

    private fun createValidFormData(): PersonalVpnFormData =
        PersonalVpnFormData(
            enabled = true,
            name = VALID_NAME,
            privateKey = VALID_PRIVATE_KEY,
            tunnelIps = setOf(VALID_TUNNEL_IP),
            peers = listOf(createValidPeerFormData()),
        )

    private fun createValidPeerFormData(): PeerFormData =
        PeerFormData(
            id = "valid-peer-id",
            publicKey = VALID_PUBLIC_KEY,
            allowedIps = setOf(VALID_ALLOWED_IP),
            endpointAddress = VALID_ENDPOINT_HOST,
            endpointPort = VALID_ENDPOINT_PORT.toString(),
        )
}
