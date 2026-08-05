package net.mullvad.mullvadvpn.test.e2e.misc

import co.touchlab.kermit.Logger
import kotlinx.coroutines.runBlocking
import net.mullvad.mullvadvpn.test.e2e.router.firewall.FirewallClient
import org.junit.jupiter.api.extension.AfterTestExecutionCallback
import org.junit.jupiter.api.extension.BeforeTestExecutionCallback
import org.junit.jupiter.api.extension.ExtendWith
import org.junit.jupiter.api.extension.ExtensionContext

@Retention(AnnotationRetention.RUNTIME)
@ExtendWith(ClearFirewallRules.ClearFirewallRulesAfterTest::class)
annotation class ClearFirewallRules {
    class ClearFirewallRulesAfterTest : BeforeTestExecutionCallback, AfterTestExecutionCallback {
        val firewallClient = FirewallClient()

        override fun beforeTestExecution(context: ExtensionContext?) {
            runBlocking {
                try {
                    firewallClient.removeAllRules()
                } catch (e: Exception) {
                    // Ignore any exceptions
                    Logger.d("firewallClient.removeAllRules failed in beforeTestExecution", e)
                }
            }
        }

        override fun afterTestExecution(context: ExtensionContext?) {
            runBlocking {
                try {
                    firewallClient.removeAllRules()
                } catch (e: Exception) {
                    // Ignore any exceptions
                    Logger.d("firewallClient.removeAllRules failed in afterTestExecution", e)
                }
            }
        }
    }
}
