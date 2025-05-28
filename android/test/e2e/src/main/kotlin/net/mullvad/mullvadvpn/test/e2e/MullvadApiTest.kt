package net.mullvad.mullvadvpn.test.e2e

import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.test.api.misc.AccountProvider
import net.mullvad.mullvadvpn.test.api.mullvad.MullvadApi
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Disabled
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertDoesNotThrow

@Disabled("Only used developing the MullvadApi")
class MullvadApiTest {
    private val mullvadApi = MullvadApi(BuildConfig.INFRASTRUCTURE_BASE_DOMAIN)
    private val accountProvider =
        AccountProvider.createAccountProvider(
            infrastructure = BuildConfig.FLAVOR_infrastructure,
            baseDomain = BuildConfig.INFRASTRUCTURE_BASE_DOMAIN,
        )

    @Test
    fun testLogin() = runTest {
        val validAccountNumber = accountProvider.getValidAccountNumber()
        val accessToken = assertDoesNotThrow { mullvadApi.login(validAccountNumber) }
        assertTrue(accessToken.isNotBlank())
    }

    @Test
    fun testGetDeviceList() = runTest {
        val validAccountNumber = accountProvider.getValidAccountNumber()
        val accessToken = assertDoesNotThrow { mullvadApi.login(validAccountNumber) }
        assertTrue(accessToken.isNotBlank())
        assertDoesNotThrow { mullvadApi.getDeviceList(accessToken) }
    }
}
