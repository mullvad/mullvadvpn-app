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

/**
 * Should be applied to any test class that has a test subject that uses the main dispatcher. This
 * is the default dispatcher for coroutines on Android and is for example used by the
 * viewModelScope. This avoids test flakiness due to the test dispatcher eagerly starting coroutines
 * instead of waiting for runCurrent for them to start.
 */
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
