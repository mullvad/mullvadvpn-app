package net.mullvad.mullvadvpn.test.e2e.api.mullvad

import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.test.e2e.misc.AccountProvider
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Disabled
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertDoesNotThrow

@Disabled("Only used developing the MullvadApi")
class MullvadApiTest {
    private val mullvadApi = MullvadApi()

    @Test
    fun testLogin() = runTest {
        val validAccountNumber = AccountProvider.getValidAccountNumber()
        val accessToken = assertDoesNotThrow { mullvadApi.login(validAccountNumber) }
        assert(accessToken.isNotEmpty())
    }

    @Test
    fun testGetDeviceList() = runTest {
        val validAccountNumber = AccountProvider.getValidAccountNumber()
        val accessToken = assertDoesNotThrow { mullvadApi.login(validAccountNumber) }
        assertTrue(accessToken.isNotBlank())

        assertDoesNotThrow { mullvadApi.getDeviceList(accessToken) }
    }
}
