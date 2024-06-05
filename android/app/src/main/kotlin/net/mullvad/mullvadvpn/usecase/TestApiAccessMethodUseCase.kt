package net.mullvad.mullvadvpn.usecase

import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.flow
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodType
import net.mullvad.mullvadvpn.lib.model.TestApiAccessMethodState
import net.mullvad.mullvadvpn.repository.ApiAccessRepository

class TestApiAccessMethodUseCase(private val apiAccessRepository: ApiAccessRepository) {
    suspend fun testApiAccessMethod(input: TestApiAccessMethodInput) = flow {
        emit(TestApiAccessMethodState.Testing)
        when (input) {
            is TestApiAccessMethodInput.TestExistingMethod ->
                apiAccessRepository
                    .testApiAccessMethodById(input.apiAccessMethodId)
                    .fold(
                        { emit(TestApiAccessMethodState.Result.Failure) },
                        { emit(TestApiAccessMethodState.Result.Successful) }
                    )
            is TestApiAccessMethodInput.TestNewMethod ->
                apiAccessRepository
                    .testCustomApiAccessMethod(input.customProxy)
                    .fold(
                        { emit(TestApiAccessMethodState.Result.Failure) },
                        { emit(TestApiAccessMethodState.Result.Successful) }
                    )
        }
        delay(TEST_METHOD_RESULT_TIME_DURATION)
        emit(null)
    }

    companion object {
        private val TEST_METHOD_RESULT_TIME_DURATION = 5.seconds
    }
}

sealed interface TestApiAccessMethodInput {
    data class TestNewMethod(val customProxy: ApiAccessMethodType.CustomProxy) :
        TestApiAccessMethodInput

    data class TestExistingMethod(val apiAccessMethodId: ApiAccessMethodId) :
        TestApiAccessMethodInput
}
