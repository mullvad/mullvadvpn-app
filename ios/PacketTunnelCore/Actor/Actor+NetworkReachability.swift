//
//  Actor+NetworkReachability.swift
//  PacketTunnelCore
//
//  Created by pronebird on 26/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension PacketTunnelActor {
    /**
     Start observing changes to default path.

     - Parameter notifyObserverWithCurrentPath: notifies path observer with the current path when set to `true`.
     */
    func startDefaultPathObserver(notifyObserverWithCurrentPath: Bool = false) {
        defaultPathObserver.start { [weak self] networkPath in
            guard let self else { return }
            Task { await self.enqueueDefaultPathChange(networkPath) }
        }

        if notifyObserverWithCurrentPath, let currentPath = defaultPathObserver.defaultPath {
            Task { await self.enqueueDefaultPathChange(currentPath) }
        }
    }

    /// Stop observing changes to default path.
    func stopDefaultPathObserver() {
        defaultPathObserver.stop()
    }

    /**
     Event handler that receives new network path and schedules it for processing on task queue to avoid interlacing with other tasks.

     - Important: Internal implementation must not call this method from operation executing on `TaskQueue`.

     - Parameter networkPath: new default path
     */
    private func enqueueDefaultPathChange(_ networkPath: NetworkPath) async {
        try? await taskQueue.add(kind: .networkReachability) { [self] in
            let newReachability = networkPath.networkReachability

            func mutateConnectionState(_ connState: inout ConnectionState) -> Bool {
                if connState.networkReachability != newReachability {
                    connState.networkReachability = newReachability
                    return true
                }
                return false
            }

            switch state {
            case var .connecting(connState):
                if mutateConnectionState(&connState) {
                    state = .connecting(connState)
                }

            case var .connected(connState):
                if mutateConnectionState(&connState) {
                    state = .connected(connState)
                }

            case var .reconnecting(connState):
                if mutateConnectionState(&connState) {
                    state = .reconnecting(connState)
                }

            case var .disconnecting(connState):
                if mutateConnectionState(&connState) {
                    state = .disconnecting(connState)
                }

            case var .error(blockedState):
                if blockedState.networkReachability != newReachability {
                    blockedState.networkReachability = newReachability
                    state = .error(blockedState)
                }

            case .initial, .disconnected:
                break
            }
        }
    }
}
