package net.mullvad.mullvadvpn.feature.customvpn.impl.edit

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import arrow.core.Either
import arrow.core.left
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import java.net.InetSocketAddress
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlinx.coroutines.cancel
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
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
import net.mullvad.mullvadvpn.lib.common.validator.IpValidator
import net.mullvad.mullvadvpn.lib.model.CustomVpnConfig
import net.mullvad.mullvadvpn.lib.model.KeyParseError
import net.mullvad.mullvadvpn.lib.model.ParsePortError
import net.mullvad.mullvadvpn.lib.model.PeerConfig
import net.mullvad.mullvadvpn.lib.model.SetCustomVpnConfigError
import net.mullvad.mullvadvpn.lib.model.TunnelConfig
import net.mullvad.mullvadvpn.lib.model.WireguardKey
import net.mullvad.mullvadvpn.lib.repository.CustomVpnRepository
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class CustomVpnEditViewModelTest {

    private val mockCustomVpnRepository: CustomVpnRepository = mockk()
    private val ipValidator = IpValidator()

    private val configUpdates =
        MutableSharedFlow<CustomVpnConfig?>(
            replay = 1,
            onBufferOverflow = BufferOverflow.DROP_OLDEST,
        )

    private lateinit var viewModel: CustomVpnEditViewModel

    @BeforeEach
    fun setup() {
        every { mockCustomVpnRepository.configFlow } returns configUpdates
        coEvery { mockCustomVpnRepository.fetchConfig() } returns Either.Right(null)

        viewModel =
            CustomVpnEditViewModel(
                customVpnRepository = mockCustomVpnRepository,
                ipValidator = ipValidator,
                isNewConfig = false,
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
            assertIs<Lc.Loading<Unit>>(initial)
        }
    }

    @Test
    fun `when settings updates emit null custom vpn config uiState should emit default empty form data`() =
        runTest {
            // Act, Assert
            viewModel.uiState.test {
                assertEquals(Lc.Loading(Unit), awaitItem())

                initSettings(customVpnConfig = null)

                val content = awaitItem()
                assertIs<Lc.Content<CustomVpnEditUiState>>(content)
                assertFormDataMatches(FormData(), content.value.formData)
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
                assertEquals(Lc.Loading(Unit), awaitItem())

                initSettings(customVpnConfig = customVpnConfig)

                val content = awaitItem()
                assertIs<Lc.Content<CustomVpnEditUiState>>(content)
                assertFormDataMatches(
                    FormData.from(customVpnConfig),
                    content.value.formData,
                )
                assertNull(content.value.formErrors)
            }
        }

    @Test
    fun `clearConfig should call management service clearCustomVpn and reset formData`() = runTest {
        // Arrange
        val customVpnConfig = createSampleCustomVpnConfig()
        coEvery { mockCustomVpnRepository.clearConfig() } coAnswers
            {
                initSettings(null)
                Unit.right()
            }

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(Lc.Loading(Unit), awaitItem())

            initSettings(customVpnConfig = customVpnConfig)

            assertIs<Lc.Content<CustomVpnEditUiState>>(awaitItem())

            viewModel.clearConfig()

            val updatedContent = awaitItem()
            assertIs<Lc.Content<CustomVpnEditUiState>>(updatedContent)
            assertFormDataMatches(FormData(), updatedContent.value.formData)
        }

        coVerify(exactly = 1) { mockCustomVpnRepository.clearConfig() }
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
                assertEquals(CustomVpnSideEffect.FormErrors.FieldError, awaitItem())
            }

            viewModel.uiState.test {
                val content = awaitItem()
                assertIs<Lc.Content<CustomVpnEditUiState>>(content)
                assertEquals(FormDataError.EmptyName, content.value.formErrors?.nameError)
            }

            coVerify(exactly = 0) { mockCustomVpnRepository.setConfig(any()) }
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
                assertEquals(CustomVpnSideEffect.FormErrors.FieldError, awaitItem())
            }

            viewModel.uiState.test {
                val content = awaitItem()
                assertIs<Lc.Content<CustomVpnEditUiState>>(content)
                val keyError = content.value.formErrors?.privateKeyError
                assertNotNull(keyError)
                assertEquals(invalidPrivateKey, keyError.errorValue)
                assertIs<KeyParseError.InvalidLength>(keyError.keyParseError)
            }

            coVerify(exactly = 0) { mockCustomVpnRepository.setConfig(any()) }
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
                assertEquals(CustomVpnSideEffect.FormErrors.FieldError, awaitItem())
            }

            viewModel.uiState.test {
                val content = awaitItem()
                assertIs<Lc.Content<CustomVpnEditUiState>>(content)
                val ipError = content.value.formErrors?.tunnelIpErrors?.get(invalidIp)
                assertEquals(FormDataError.TunnelIp(invalidIp), ipError)
            }

            coVerify(exactly = 0) { mockCustomVpnRepository.setConfig(any()) }
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
                assertEquals(CustomVpnSideEffect.FormErrors.FieldError, awaitItem())
            }

            viewModel.uiState.test {
                val content = awaitItem()
                assertIs<Lc.Content<CustomVpnEditUiState>>(content)
                val peerError = content.value.formErrors?.peerErrors?.get(peerId)
                assertNotNull(peerError)
                val keyError = peerError.publicKeyError
                assertNotNull(keyError)
                assertEquals(invalidPublicKey, keyError.errorValue)
                assertIs<KeyParseError.InvalidLength>(keyError.keyParseError)
            }

            coVerify(exactly = 0) { mockCustomVpnRepository.setConfig(any()) }
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
                assertEquals(CustomVpnSideEffect.FormErrors.FieldError, awaitItem())
            }

            viewModel.uiState.test {
                val content = awaitItem()
                assertIs<Lc.Content<CustomVpnEditUiState>>(content)
                val peerError = content.value.formErrors?.peerErrors?.get(peerId)
                assertNotNull(peerError)
                assertEquals(
                    FormDataError.AllowedIp(invalidAllowedIp),
                    peerError.allowedIpErrors[invalidAllowedIp],
                )
            }

            coVerify(exactly = 0) { mockCustomVpnRepository.setConfig(any()) }
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
                assertEquals(CustomVpnSideEffect.FormErrors.FieldError, awaitItem())
            }

            viewModel.uiState.test {
                val content = awaitItem()
                assertIs<Lc.Content<CustomVpnEditUiState>>(content)
                val peerError = content.value.formErrors?.peerErrors?.get(peerId)
                assertNotNull(peerError)
                assertEquals(
                    FormDataError.EndpointAddress(invalidEndpoint),
                    peerError.endpointAddressError,
                )
            }

            coVerify(exactly = 0) { mockCustomVpnRepository.setConfig(any()) }
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
                assertEquals(CustomVpnSideEffect.FormErrors.FieldError, awaitItem())
            }

            viewModel.uiState.test {
                val content = awaitItem()
                assertIs<Lc.Content<CustomVpnEditUiState>>(content)
                val peerError = content.value.formErrors?.peerErrors?.get(peerId)
                assertNotNull(peerError)
                val portError = peerError.endpointPortError
                assertNotNull(portError)
                assertEquals(invalidPort, portError.errorValue)
                assertIs<ParsePortError.NotANumber>(portError.parsePortError)
            }

            coVerify(exactly = 0) { mockCustomVpnRepository.setConfig(any()) }
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
                assertEquals(CustomVpnSideEffect.FormErrors.FieldError, awaitItem())
            }

            viewModel.uiState.test {
                val content = awaitItem()
                assertIs<Lc.Content<CustomVpnEditUiState>>(content)
                val peerError = content.value.formErrors?.peerErrors?.get(peerId)
                assertNotNull(peerError)
                val portError = peerError.endpointPortError
                assertNotNull(portError)
                assertEquals(outOfRangePort, portError.errorValue)
                assertIs<ParsePortError.OutOfRange>(portError.parsePortError)
            }

            coVerify(exactly = 0) { mockCustomVpnRepository.setConfig(any()) }
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
                            ),
                        ),
                    enabled = true,
                    name = VALID_NAME,
                )
            coEvery { mockCustomVpnRepository.setConfig(expectedConfig) } coAnswers
                {
                    initSettings(expectedConfig)
                    Unit.right()
                }

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.save(validFormData)
                assertEquals(CustomVpnSideEffect.ConfigurationSaved, awaitItem())
            }

            viewModel.uiState.test {
                val content = awaitItem()
                assertIs<Lc.Content<CustomVpnEditUiState>>(content)
                assertFormDataMatches(
                    FormData.from(expectedConfig),
                    content.value.formData,
                )
                assertNull(content.value.formErrors)
            }

            coVerify(exactly = 1) { mockCustomVpnRepository.setConfig(expectedConfig) }
        }

    @Test
    fun `when management service fails to set custom vpn config should emit FailedToSave side effect`() =
        runTest {
            // Arrange
            initSettings()
            val validFormData = createValidFormData()
            val saveError = SetCustomVpnConfigError.Unknown(RuntimeException("Save failed"))
            coEvery { mockCustomVpnRepository.setConfig(any()) } returns saveError.left()

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.save(validFormData)
                val sideEffect = awaitItem()
                assertIs<CustomVpnSideEffect.FailedToSave>(sideEffect)
                assertEquals(saveError.toString(), sideEffect.reason)
            }

            coVerify(exactly = 1) { mockCustomVpnRepository.setConfig(any()) }
        }

    private fun initSettings(customVpnConfig: CustomVpnConfig? = null) {
        configUpdates.tryEmit(customVpnConfig)
    }

    private fun assertFormDataMatches(
        expected: FormData,
        actual: FormData,
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

    private fun createValidFormData(): FormData =
        FormData(
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
