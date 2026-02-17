package net.mullvad.mullvadvpn.lib.grpc

import io.grpc.ManagedChannel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.asExecutor
import mullvad_daemon.relay_selector.RelaySelectorServiceGrpcKt
import net.mullvad.mullvadvpn.lib.grpc.mapper.fromDomain
import net.mullvad.mullvadvpn.lib.grpc.mapper.toDomain
import net.mullvad.mullvadvpn.lib.grpc.util.LogInterceptor
import net.mullvad.mullvadvpn.lib.model.RelayPartitions
import net.mullvad.mullvadvpn.lib.model.RelaySelectorPredicate

@Suppress("TooManyFunctions", "LargeClass")
class RelaySelectorService(
    private val channel: ManagedChannel,
    private val extensiveLogging: Boolean,
    private val scope: CoroutineScope,
) {
    private var job: Job? = null

    private val service by lazy {
        RelaySelectorServiceGrpcKt.RelaySelectorServiceCoroutineStub(channel)
            .withExecutor(Dispatchers.IO.asExecutor())
            .let {
                if (extensiveLogging) {
                    it.withInterceptors(LogInterceptor())
                } else it
            }
            .withWaitForReady()
    }

    suspend fun partitionRelays(predicate: RelaySelectorPredicate): RelayPartitions {
        return service.partitionRelays(predicate.fromDomain()).toDomain()
    }
}
