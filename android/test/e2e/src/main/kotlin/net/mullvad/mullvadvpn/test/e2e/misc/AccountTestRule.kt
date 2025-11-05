package net.mullvad.mullvadvpn.test.e2e.misc

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.extension.AfterEachCallback
import org.junit.jupiter.api.extension.BeforeEachCallback
import org.junit.jupiter.api.extension.ExtensionContext

class AccountTestRule(val withTime: Boolean = true) : BeforeEachCallback, AfterEachCallback {
    lateinit var validAccountNumber: String
    lateinit var invalidAccountNumber: String

    override fun beforeEach(context: ExtensionContext): Unit = runBlocking {
        validAccountNumber = AccountProvider.getValidAccountNumber(withTime)
        invalidAccountNumber = AccountProvider.getInvalidAccountNumber()
    }

    override fun afterEach(context: ExtensionContext?): Unit = runBlocking {
        AccountProvider.tryDeletePartnerAccount(validAccountNumber)
    }
}
