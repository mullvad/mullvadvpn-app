package net.mullvad.mullvadvpn.feature.dns.impl

import androidx.compose.material3.SnackbarHostState
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollToNode
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.feature.dns.impl.CustomDnsEntry
import net.mullvad.mullvadvpn.feature.dns.impl.DnsSettingsScreen
import net.mullvad.mullvadvpn.feature.dns.impl.DnsSettingsUiState
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_LAST_ITEM_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_VPN_SETTINGS_TEST_TAG
import net.mullvad.mullvadvpn.screen.test.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.screen.test.setContentWithTheme
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class DnsSettingsScreenTest {
    @OptIn(ExperimentalTestApi::class)
    @JvmField
    @RegisterExtension
    val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    private fun createDefaultUiState(
        isModal: Boolean = false,
        contentBlockersExpanded: Boolean = false,
        contentBlockersEnabled: Boolean = true,
        defaultDnsOptions: DefaultDnsOptions = DefaultDnsOptions(),
        customDnsEnabled: Boolean = false,
        customDnsEntries: List<CustomDnsEntry> = emptyList(),
        showUnreachableLocalDnsWarning: Boolean = false,
        showUnreachableIpv6DnsWarning: Boolean = false,
    ) =
        DnsSettingsUiState(
            isModal = isModal,
            contentBlockersExpanded = contentBlockersExpanded,
            contentBlockersEnabled = contentBlockersEnabled,
            defaultDnsOptions = defaultDnsOptions,
            customDnsEnabled = customDnsEnabled,
            customDnsEntries = customDnsEntries,
            showUnreachableLocalDnsWarning = showUnreachableLocalDnsWarning,
            showUnreachableIpv6DnsWarning = showUnreachableIpv6DnsWarning,
        )

    private fun ComposeContext.initScreen(
        state: Lc<Unit, DnsSettingsUiState> = createDefaultUiState().toLc(),
        navigateToDns: (index: Int?, address: String?) -> Unit = { _, _ -> },
        navigateToCustomDnsInfo: () -> Unit = {},
        onToggleDnsClick: (Boolean) -> Unit = {},
        onToggleBlockAds: (Boolean) -> Unit = {},
        onToggleBlockMalware: (Boolean) -> Unit = {},
        onToggleBlockGambling: (Boolean) -> Unit = {},
        onToggleBlockTrackers: (Boolean) -> Unit = {},
        onToggleBlockSocialMedia: (Boolean) -> Unit = {},
        onToggleBlockAdultContent: (Boolean) -> Unit = {},
        onToggleAllContentBlockers: (Boolean) -> Unit = {},
        onToggleContentBlockersExpanded: () -> Unit = {},
        navigateToMalwareInfo: () -> Unit = {},
        navigateToContentBlockersInfo: () -> Unit = {},
        onBackClick: () -> Unit = {},
    ) {
        setContentWithTheme {
            DnsSettingsScreen(
                state = state,
                modifier = Modifier,
                snackbarHostState = SnackbarHostState(),
                navigateToDns = navigateToDns,
                navigateToCustomDnsInfo = navigateToCustomDnsInfo,
                onToggleDnsClick = onToggleDnsClick,
                onToggleBlockAds = onToggleBlockAds,
                onToggleBlockMalware = onToggleBlockMalware,
                onToggleBlockGambling = onToggleBlockGambling,
                onToggleBlockTrackers = onToggleBlockTrackers,
                onToggleBlockSocialMedia = onToggleBlockSocialMedia,
                onToggleBlockAdultContent = onToggleBlockAdultContent,
                onToggleAllContentBlockers = onToggleAllContentBlockers,
                onToggleContentBlockersExpanded = onToggleContentBlockersExpanded,
                navigateToMalwareInfo = navigateToMalwareInfo,
                navigateToContentBlockersInfo = navigateToContentBlockersInfo,
                onBackClick = onBackClick,
            )
        }
    }

    @Test
    fun testCustomDnsAddressesAndAddButtonVisibleWhenCustomDnsEnabled() = composeExtension.use {
        // Arrange
        initScreen(
            state =
                createDefaultUiState(
                        customDnsEnabled = true,
                        customDnsEntries =
                            listOf(
                                CustomDnsEntry(
                                    address = DUMMY_DNS_ADDRESS,
                                    isLocal = false,
                                    isIpv6 = false,
                                ),
                                CustomDnsEntry(
                                    address = DUMMY_DNS_ADDRESS_2,
                                    isLocal = false,
                                    isIpv6 = false,
                                ),
                                CustomDnsEntry(
                                    address = DUMMY_DNS_ADDRESS_3,
                                    isLocal = false,
                                    isIpv6 = false,
                                ),
                            ),
                    )
                    .toLc()
        )

        // Assert
        onNodeWithText(DUMMY_DNS_ADDRESS).assertExists()
        onNodeWithText(DUMMY_DNS_ADDRESS_2).assertExists()
        onNodeWithText(DUMMY_DNS_ADDRESS_3).assertExists()
        onNodeWithText("Add a server").assertExists()
    }

    @Test
    fun testCustomDnsAddressesAndAddButtonNotVisibleWhenCustomDnsDisabled() = composeExtension.use {
        // Arrange
        initScreen(
            state =
                createDefaultUiState(
                        customDnsEnabled = false,
                        customDnsEntries =
                            listOf(
                                CustomDnsEntry(
                                    address = DUMMY_DNS_ADDRESS,
                                    isLocal = false,
                                    isIpv6 = false,
                                )
                            ),
                    )
                    .toLc()
        )
        onNodeWithTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
            .performScrollToNode(hasTestTag(LAZY_LIST_LAST_ITEM_TEST_TAG))
        // Assert
        onNodeWithText(DUMMY_DNS_ADDRESS).assertDoesNotExist()
        onNodeWithText("Add a server").assertDoesNotExist()
    }

    @Test
    fun testLanWarningNotShownWhenLanTrafficEnabledAndLocalAddressIsUsed() = composeExtension.use {
        // Arrange
        initScreen(
            state =
                createDefaultUiState(
                        customDnsEnabled = true,
                        showUnreachableLocalDnsWarning = true,
                        customDnsEntries =
                            listOf(
                                CustomDnsEntry(
                                    address = DUMMY_DNS_ADDRESS,
                                    isLocal = true,
                                    isIpv6 = false,
                                )
                            ),
                    )
                    .toLc()
        )

        // Assert
        onNodeWithContentDescription(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
    }

    @Test
    fun testLanWarningNotShowedWhenLanTrafficDisabledAndLocalAddressIsNotUsed() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    createDefaultUiState(
                            customDnsEnabled = true,
                            customDnsEntries =
                                listOf(
                                    CustomDnsEntry(
                                        address = DUMMY_DNS_ADDRESS,
                                        isLocal = false,
                                        isIpv6 = false,
                                    )
                                ),
                        )
                        .toLc()
            )

            // Assert
            onNodeWithContentDescription(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
        }

    @Test
    fun testLanWarningNotShowedWhenLanTrafficEnabledAndLocalAddressIsNotUsed() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    createDefaultUiState(
                            customDnsEnabled = true,
                            customDnsEntries =
                                listOf(
                                    CustomDnsEntry(
                                        address = DUMMY_DNS_ADDRESS,
                                        isLocal = false,
                                        isIpv6 = false,
                                    )
                                ),
                        )
                        .toLc()
            )

            // Assert
            onNodeWithContentDescription(LOCAL_DNS_SERVER_WARNING).assertDoesNotExist()
        }

    @Test
    fun testLanWarningShowedWhenAllowLanEnabledAndLocalDnsAddressIsUsed() = composeExtension.use {
        // Arrange
        initScreen(
            state =
                createDefaultUiState(
                        customDnsEnabled = true,
                        customDnsEntries =
                            listOf(
                                CustomDnsEntry(
                                    address = DUMMY_DNS_ADDRESS,
                                    isLocal = true,
                                    isIpv6 = false,
                                )
                            ),
                    )
                    .toLc()
        )

        // Assert
        onNodeWithContentDescription(LOCAL_DNS_SERVER_WARNING).assertExists()
    }

    @Test
    fun testClickAddDns() = composeExtension.use {
        // Arrange
        val mockedClickHandler: (Int?, String?) -> Unit = mockk(relaxed = true)
        initScreen(
            state =
                createDefaultUiState(
                        customDnsEnabled = true,
                        customDnsEntries =
                            listOf(CustomDnsEntry("1.1.1.1", isLocal = false, isIpv6 = false)),
                    )
                    .toLc(),
            navigateToDns = mockedClickHandler,
        )

        // Act
        onNodeWithText("Add a server").performClick()

        // Assert
        verify { mockedClickHandler.invoke(null, null) }
    }

    companion object {
        private const val LOCAL_DNS_SERVER_WARNING =
            "The local DNS server will not work unless you enable " +
                "\"Local Network Sharing\" under VPN settings."
        private const val DUMMY_DNS_ADDRESS = "0.0.0.1"
        private const val DUMMY_DNS_ADDRESS_2 = "0.0.0.2"
        private const val DUMMY_DNS_ADDRESS_3 = "0.0.0.3"
    }
}
