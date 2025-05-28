package net.mullvad.mullvadvpn.test.benchmark.rule

import kotlinx.coroutines.runBlocking
import net.mullvad.mullvadvpn.test.api.misc.AccountProvider
import net.mullvad.mullvadvpn.test.benchmark.BuildConfig
import org.junit.jupiter.api.extension.AfterEachCallback
import org.junit.jupiter.api.extension.BeforeEachCallback
import org.junit.jupiter.api.extension.ExtensionContext

class AccountTestRule : BeforeEachCallback, AfterEachCallback {
    private val accountProvider =
        AccountProvider.createAccountProvider(
            infrastructure = BuildConfig.FLAVOR,
            baseDomain = BuildConfig.INFRASTRUCTURE_BASE_DOMAIN,
        )
    lateinit var validAccountNumber: String

    override fun beforeEach(context: ExtensionContext): Unit = runBlocking {
        validAccountNumber = accountProvider.getValidAccountNumber(true)
    }

    override fun afterEach(context: ExtensionContext?): Unit = runBlocking {
        accountProvider.cleanup(validAccountNumber)
    }
}
