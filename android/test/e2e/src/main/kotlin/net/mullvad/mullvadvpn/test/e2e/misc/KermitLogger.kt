package net.mullvad.mullvadvpn.test.e2e.misc

import co.touchlab.kermit.Logger as KLogger
import io.ktor.client.plugins.logging.Logger

class KermitLogger : Logger {
    private val logger = KLogger.withTag("HttpClient")

    private val uuidRegex =
        Regex(
            "^[0-9a-fA-F]{8}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{12}$"
        )
    private val accountNumberRegex = Regex("\\d{16}")

    override fun log(message: String) {

        val redactedMessage =
            message
                .replace(accountNumberRegex, "<REDACTED_ACCOUNT_NUMBER>")
                .replace(uuidRegex, "<REDACTED_UUID>")
        logger.d(redactedMessage)
    }
}
