package net.mullvad.mullvadvpn.test.e2e.api.partner

import androidx.test.platform.app.InstrumentationRegistry
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.test.e2e.constant.getPartnerAuth
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Disabled
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertDoesNotThrow

@Disabled("Only used developing the PartnerApi")
class PartnerApiTest {
    private val partnerApi = PartnerApi(InstrumentationRegistry.getArguments().getPartnerAuth()!!)

    @Test
    fun testCreateAccount() = runTest {
        val accessToken = assertDoesNotThrow { partnerApi.createAccount() }
        assertTrue(accessToken.isNotBlank())
    }

    @Test
    fun testAddTime() = runTest {
        val accessToken = assertDoesNotThrow { partnerApi.createAccount() }
        assertDoesNotThrow { partnerApi.addTime(accessToken, 1) }
    }

    @Test
    fun testDeleteAccount() = runTest {
        val accessToken = assertDoesNotThrow { partnerApi.createAccount() }
        assertDoesNotThrow { partnerApi.deleteAccount(accessToken) }
    }
}
