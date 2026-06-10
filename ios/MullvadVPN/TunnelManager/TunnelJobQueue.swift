//
//  TunnelJobQueue.swift
//  MullvadVPN
//
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Operations
import PacketTunnelCore

/// The narrow slice of `TunnelManager` the coordinator needs. Kept small so the coordinator
/// stays unit-testable and free of a strong reference cycle with its owner.
protocol TunnelJobQueueInteractor: Sendable {
    var tunnel: (any TunnelProtocol)? { get }

    func setTunnelStatus(_ block: @Sendable (inout TunnelStatus) -> Void)
    func startPollingTunnelStatus()

    @MainActor func didReconnectTunnel(error: Error?)
}

/// Owns tunnel-manipulation flows migrated off the legacy operation queue.
///
/// ## Exclusion model
///
/// Actors are reentrant: isolation does NOT serialize a method across its `await` suspension
/// points, so two `reconnect` calls would interleave. Exclusion is therefore explicit — each
/// job awaits the previous job's task handle (`lastJob` chaining). This chain is the
/// replacement for the legacy `MutuallyExclusive` condition and the pattern other flows
/// (start/stop/setAccount) should adopt as they migrate.
///
/// While legacy operations remain, each job also acquires a placeholder operation in the
/// legacy `manageTunnel` exclusivity category so coordinator jobs cannot overlap unmigrated
/// start/stop/setAccount operations.
@available(iOSApplicationExtension, unavailable)
actor TunnelJobQueue {
    private let interactor: any TunnelJobQueueInteractor
    private let operationQueue: AsyncOperationQueue
    private let backgroundTaskProvider: BackgroundTaskProviding
    private let legacyExclusivityCategory: String

    /// Tail of the job chain. Mutated only synchronously within the actor (no `await`
    /// between read and write), which closes the reentrancy hole.
    private var lastJob: Task<Void, Error>?

    init(
        interactor: any TunnelJobQueueInteractor,
        operationQueue: AsyncOperationQueue,
        backgroundTaskProvider: BackgroundTaskProviding,
        legacyExclusivityCategory: String
    ) {
        self.interactor = interactor
        self.operationQueue = operationQueue
        self.backgroundTaskProvider = backgroundTaskProvider
        self.legacyExclusivityCategory = legacyExclusivityCategory
    }

    /// Request the packet tunnel to reconnect, optionally to a new relay selection.
    ///
    /// Serialized against all other coordinator jobs and against legacy `manageTunnel`
    /// operations. Cancelling the calling task abandons the wait but does not abort a job
    /// that has already started, matching the legacy fire-and-forget behavior.
    func reconnect(selectNewRelay: Bool) async throws {
        let previousJob = lastJob
        let job = Task {
            // Wait for the previous job regardless of how it ended.
            _ = await previousJob?.result

            try await self.runExclusively {
                try await self.performReconnect(selectNewRelay: selectNewRelay)
            }
        }
        lastJob = job

        try await job.value
    }

    /// Run `body` while holding a placeholder operation in the legacy exclusivity category.
    // TODO(operation-migration): remove this bridge once start/stop/setAccount run on the
    // coordinator; `lastJob` chaining then carries the exclusion alone.
    private func runExclusively<Success: Sendable>(
        _ body: @escaping @Sendable () async throws -> Success
    ) async throws -> Success {
        let releaseExclusivity = await acquireLegacyExclusivity()
        defer { releaseExclusivity() }

        return try await body()
    }

    /// Enqueue a placeholder operation in the legacy category and suspend until it starts
    /// executing — at which point the category is exclusively ours. The returned closure
    /// finishes the placeholder, releasing the category.
    private func acquireLegacyExclusivity() async -> @Sendable () -> Void {
        await withCheckedContinuation { continuation in
            let operation = AsyncBlockOperation(dispatchQueue: nil) { finish -> Cancellable in
                continuation.resume(returning: { finish(nil) })
                return AnyCancellable()
            }
            operation.addCondition(MutuallyExclusive(category: legacyExclusivityCategory))
            operationQueue.addOperation(operation)
        }
    }

    private func performReconnect(selectNewRelay: Bool) async throws {
        interactor.startPollingTunnelStatus()

        do {
            try await withBackgroundTask(
                name: "Reconnect tunnel",
                provider: backgroundTaskProvider
            ) {
                guard let tunnel = self.interactor.tunnel else {
                    throw UnsetTunnelError()
                }

                let observedState = try await tunnel.reconnectTunnel(
                    to: selectNewRelay ? .random : .current
                )

                if let connectionState = observedState.connectionState {
                    // This makes the app feel very responsive when the user wants to reconnect.
                    // If the tunnel is already connected, at worst the next tunnel status poll
                    // will correct the state.
                    self.interactor.setTunnelStatus { tunnelStatus in
                        tunnelStatus.state = .reconnecting(
                            connectionState.selectedRelays,
                            isPostQuantum: connectionState.isPostQuantum,
                            isDaita: connectionState.isDaitaEnabled
                        )
                        tunnelStatus.observedState = observedState
                    }
                }
            }

            await interactor.didReconnectTunnel(error: nil)
        } catch {
            await interactor.didReconnectTunnel(error: error)
            throw error
        }
    }
}
