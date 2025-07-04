package net.mullvad.mullvadvpn.test.e2e.misc

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.extension.BeforeEachCallback
import org.junit.jupiter.api.extension.ExtensionContext

class AccountTestRule(val withTime: Boolean = true) : BeforeEachCallback {
    lateinit var validAccountNumber: String
    lateinit var invalidAccountNumber: String

    override fun beforeEach(context: ExtensionContext): Unit = runBlocking {
        validAccountNumber = AccountProvider.getValidAccountNumber(withTime)
        invalidAccountNumber = AccountProvider.getInvalidAccountNumber()
    }
}
