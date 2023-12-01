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

     - Parameter notifyObserverWithCurrentPath: immediately notifies path observer with the current path when set to `true`.
     */
    func startDefaultPathObserver(notifyObserverWithCurrentPath: Bool = false) {
        logger.trace("Start default path observer.")

        defaultPathObserver.start { [weak self] networkPath in
            self?.commandChannel.send(.networkReachability(networkPath))
        }

        if notifyObserverWithCurrentPath, let currentPath = defaultPathObserver.defaultPath {
            commandChannel.send(.networkReachability(currentPath))
        }
    }

    /// Stop observing changes to default path.
    func stopDefaultPathObserver() {
        logger.trace("Stop default path observer.")

        defaultPathObserver.stop()
    }

    /**
     Event handler that receives new network path from tunnel monitor and updates internal state with new network reachability status.

     - Parameter networkPath: new default path
     */
    func handleDefaultPathChange(_ networkPath: NetworkPath) {
        tunnelMonitor.handleNetworkPathUpdate(networkPath)

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
                Task {
                    await blockAllTrafficUntilDeviceIsConnected()
                }
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
