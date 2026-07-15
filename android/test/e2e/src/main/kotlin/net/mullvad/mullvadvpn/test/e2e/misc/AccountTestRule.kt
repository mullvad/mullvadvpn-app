package net.mullvad.mullvadvpn.test.e2e.misc

import kotlinx.coroutines.runBlocking
import net.mullvad.mullvadvpn.test.api.misc.AccountProvider
import net.mullvad.mullvadvpn.test.e2e.BuildConfig
import org.junit.jupiter.api.extension.AfterEachCallback
import org.junit.jupiter.api.extension.BeforeEachCallback
import org.junit.jupiter.api.extension.ExtensionContext

class AccountTestRule(val withTime: Boolean = true) : BeforeEachCallback, AfterEachCallback {
    private val accountProvider =
        AccountProvider.createAccountProvider(
            infrastructure = BuildConfig.FLAVOR_infrastructure,
            baseDomain = BuildConfig.INFRASTRUCTURE_BASE_DOMAIN,
        )
    lateinit var validAccountNumber: String
    lateinit var invalidAccountNumber: String

    override fun beforeEach(context: ExtensionContext): Unit = runBlocking {
        validAccountNumber = accountProvider.getValidAccountNumber(withTime)
        invalidAccountNumber =
            accountProvider.getInvalidAccountNumber(BuildConfig.FLAVOR_infrastructure)
    }

    override fun afterEach(context: ExtensionContext?): Unit = runBlocking {
        accountProvider.cleanup(validAccountNumber)
    }
}
