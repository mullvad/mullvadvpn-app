package net.mullvad.mullvadvpn.lib.common.test

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.TestDispatcher
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.setMain
import org.junit.jupiter.api.extension.AfterEachCallback
import org.junit.jupiter.api.extension.BeforeEachCallback
import org.junit.jupiter.api.extension.ExtensionContext

@OptIn(ExperimentalCoroutinesApi::class)
class TestCoroutineRule(val testDispatcher: TestDispatcher = UnconfinedTestDispatcher()) :
    BeforeEachCallback, AfterEachCallback {

    override fun beforeEach(var1: ExtensionContext?) {
        Dispatchers.setMain(testDispatcher)
    }

    override fun afterEach(var1: ExtensionContext?) {
        Dispatchers.resetMain()
        testDispatcher.scheduler.runCurrent()
    }
}
