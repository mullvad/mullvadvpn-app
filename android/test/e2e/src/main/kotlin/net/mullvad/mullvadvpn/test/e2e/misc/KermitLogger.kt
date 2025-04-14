package net.mullvad.mullvadvpn.test.e2e.misc

import co.touchlab.kermit.Logger as KLogger
import io.ktor.client.plugins.logging.Logger

class KermitLogger : Logger {
    private val logger = KLogger.withTag("HttpClient")

    override fun log(message: String) {
        logger.d(message)
    }
}
