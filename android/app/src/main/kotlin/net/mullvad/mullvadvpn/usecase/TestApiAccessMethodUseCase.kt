package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.flow
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodType
import net.mullvad.mullvadvpn.repository.ApiAccessRepository

class TestApiAccessMethodUseCase(private val apiAccessRepository: ApiAccessRepository) {
    suspend fun testApiAccessMethod(input: TestApiAccessMethodInput) = flow {
        emit(TestApiAccessMethodState.Testing)
        when (input) {
            is TestApiAccessMethodInput.TestExistingMethod ->
                apiAccessRepository
                    .testApiAccessMethodById(input.apiAccessMethodId)
                    .fold(
                        { emit(TestApiAccessMethodState.Failure) },
                        { emit(TestApiAccessMethodState.Successful) }
                    )
            is TestApiAccessMethodInput.TestNewMethod ->
                apiAccessRepository
                    .testCustomApiAccessMethod(input.customProxy)
                    .fold(
                        { emit(TestApiAccessMethodState.Failure) },
                        { emit(TestApiAccessMethodState.Successful) }
                    )
        }
        delay(TEST_METHOD_RESULT_TIME_MS)
        emit(null)
    }

    companion object {
        private const val TEST_METHOD_RESULT_TIME_MS = 1000L * 5
    }
}

sealed interface TestApiAccessMethodInput {
    data class TestNewMethod(val customProxy: ApiAccessMethodType.CustomProxy) :
        TestApiAccessMethodInput

    data class TestExistingMethod(val apiAccessMethodId: ApiAccessMethodId) :
        TestApiAccessMethodInput
}

sealed interface TestApiAccessMethodState {
    data object Testing : TestApiAccessMethodState

    data object Successful : TestApiAccessMethodState

    data object Failure : TestApiAccessMethodState
}
