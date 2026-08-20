//
//  TunnelJobQueue.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-08-19.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import MullvadTypes
import MullvadREST

protocol TunnelJobQueueInteractor: Sendable {
    var tunnel: (any TunnelProtocol)? { get }

    func setTunnelStatus(_ block: @Sendable (inout TunnelStatus) -> Void)
    func startPollingTunnelStatus()

    @MainActor func didReconnectTunnel(error: Error?)
}

actor TunnelJobQueue {
    private let interactor: any TunnelJobQueueInteractor
    private let backgroundTaskProvider: BackgroundTaskProviding
    private let exclusivityCategory: String
    private var lastJob: Task<Void, Error>?
    private let exclusivityCoordinator = AsyncExclusivityCoordinator()

    init(
        interactor: any TunnelJobQueueInteractor, backgroundTaskProvider: BackgroundTaskProviding,
        exclusivityCategory: String
    ) {
        self.interactor = interactor
        self.backgroundTaskProvider = backgroundTaskProvider
        self.exclusivityCategory = exclusivityCategory
    }

    func reconnect(selectNewRelay: Bool) async throws {
        try await exclusivityCoordinator.withExclusiveAccess {
            try await performReconnect(selectNewRelay: selectNewRelay)
        }
    }

    private func performReconnect(selectNewRelay: Bool) async throws {
        interactor.startPollingTunnelStatus()

        do {
            //            try await withBackgroundTask(
            //                name: "Reconnect tunnel",
            //                provider: backgroundTaskProvider
            //            ) {
            guard let tunnel = self.interactor.tunnel else {
                throw UnsetTunnelError()
            }

            let observedState = try await tunnel.reconnectTunnel(to: selectNewRelay ? .random : .current)

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
            //            }

            await interactor.didReconnectTunnel(error: nil)
        } catch {
            await interactor.didReconnectTunnel(error: error)
            throw error
        }
    }
}

private actor AsyncExclusivityCoordinator {
    private var isOccupied = false
    private var waiters: [CheckedContinuation<Void, Never>] = []

    func withExclusiveAccess<T: Sendable>(
        _ operation: @Sendable () async throws -> T
    ) async throws -> T {
        await acquire()

        defer {
            release()
        }

        try Task.checkCancellation()

        return try await operation()
    }

    private func acquire() async {
        guard isOccupied else {
            isOccupied = true
            return
        }

        await withCheckedContinuation { continuation in
            waiters.append(continuation)
        }
    }

    private func release() {
        guard let waiter = waiters.first else {
            isOccupied = false
            return
        }

        waiters.removeFirst()
        waiter.resume()
    }
}
