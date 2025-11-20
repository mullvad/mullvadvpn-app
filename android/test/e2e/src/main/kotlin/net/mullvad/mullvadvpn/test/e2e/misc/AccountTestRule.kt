package net.mullvad.mullvadvpn.test.e2e.misc

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.extension.AfterEachCallback
import org.junit.jupiter.api.extension.BeforeEachCallback
import org.junit.jupiter.api.extension.ExtensionContext

class AccountTestRule(val withTime: Boolean = true) : BeforeEachCallback, AfterEachCallback {
    private val accountProvider = AccountProvider.createAccountProvider()
    lateinit var validAccountNumber: String
    lateinit var invalidAccountNumber: String

    override fun beforeEach(context: ExtensionContext): Unit = runBlocking {
        validAccountNumber = accountProvider.getValidAccountNumber(withTime)
        invalidAccountNumber = accountProvider.getInvalidAccountNumber()
    }

    override fun afterEach(context: ExtensionContext?): Unit = runBlocking {
        accountProvider.cleanup(validAccountNumber)
    }
}
