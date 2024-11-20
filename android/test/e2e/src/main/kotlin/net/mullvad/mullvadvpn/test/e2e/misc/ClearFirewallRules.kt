package net.mullvad.mullvadvpn.test.e2e.misc

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.extension.AfterTestExecutionCallback
import org.junit.jupiter.api.extension.ExtendWith
import org.junit.jupiter.api.extension.ExtensionContext

@Retention(AnnotationRetention.RUNTIME)
@ExtendWith(ClearFirewallRules.ClearFirewallRulesAfterTest::class)
annotation class ClearFirewallRules {
    class ClearFirewallRulesAfterTest : AfterTestExecutionCallback {
        override fun afterTestExecution(context: ExtensionContext?) {
            runBlocking {
                val firewallClient = FirewallClient()
                firewallClient.removeAllRules()
            }
        }
    }
}
