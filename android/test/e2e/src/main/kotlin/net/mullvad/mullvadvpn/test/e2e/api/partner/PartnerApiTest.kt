package net.mullvad.mullvadvpn.test.e2e.api.partner

import androidx.test.platform.app.InstrumentationRegistry
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.test.e2e.constant.PARTNER_AUTH
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Disabled
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertDoesNotThrow

@Disabled
class PartnerApiTest {
    private val partnerApi =
        PartnerApi(InstrumentationRegistry.getArguments().getString(PARTNER_AUTH, null))

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
}
