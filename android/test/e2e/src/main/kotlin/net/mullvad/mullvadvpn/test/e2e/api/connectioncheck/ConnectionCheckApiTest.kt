package net.mullvad.mullvadvpn.test.e2e.api.connectioncheck

import kotlinx.coroutines.test.runTest
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Disabled
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertNotNull

@Disabled("Only used developing the ConnectionCheckApi")
class ConnectionCheckApiTest {
    private val connCheckApi = ConnectionCheckApi()

    @Test
    fun testConnCheck() = runTest {
        val result = connCheckApi.connectionCheck()
        assertNotNull(result)
        assertFalse(result.mullvadExitIp)
    }
}
