//
//  TunnelManager.State.swift
//  MullvadVPN
//
//  Created by pronebird on 26/01/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

protocol TunnelManagerStateDelegate: AnyObject {

    func tunnelManagerState(
        _ state: TunnelManager.State,
        didChangeLoadedConfiguration isLoadedConfiguration: Bool
    )

    func tunnelManagerState(
        _ state: TunnelManager.State,
        didChangeTunnelSettings newTunnelSettings: TunnelSettingsV2?
    )

    func tunnelManagerState(
        _ state: TunnelManager.State,
        didChangeTunnelStatus newTunnelStatus: TunnelStatus
    )

    func tunnelManagerState(
        _ state: TunnelManager.State,
        didChangeTunnelProvider newTunnelObject: Tunnel?,
        shouldRefreshTunnelState: Bool
    )
}

extension TunnelManager {

    class State {
        weak var delegate: TunnelManagerStateDelegate?
        let delegateQueue: DispatchQueue

        private let nslock = NSLock()
        private var _isLoadedConfiguration = false
        private var _tunnelSettings: TunnelSettingsV2?
        private var _tunnelObject: Tunnel?
        private var _tunnelStatus = TunnelStatus(
            isNetworkReachable: false,
            connectingDate: nil,
            state: .disconnected
        )

        var isLoadedConfiguration: Bool {
            get {
                nslock.lock()
                defer { nslock.unlock() }

                return _isLoadedConfiguration
            }
            set {
                nslock.lock()
                defer { nslock.unlock() }

                guard _isLoadedConfiguration != newValue else { return }

                _isLoadedConfiguration = newValue

                delegateQueue.async {
                    self.delegate?.tunnelManagerState(
                        self,
                        didChangeLoadedConfiguration: newValue
                    )
                }
            }
        }

        var tunnelSettings: TunnelSettingsV2? {
            get {
                nslock.lock()
                defer { nslock.unlock() }

                return _tunnelSettings
            }
            set {
                nslock.lock()
                defer { nslock.unlock() }

                guard _tunnelSettings != newValue else { return }

                _tunnelSettings = newValue

                delegateQueue.async {
                    self.delegate?.tunnelManagerState(self, didChangeTunnelSettings: newValue)
                }
            }
        }

        var tunnel: Tunnel? {
            nslock.lock()
            defer { nslock.unlock() }

            return _tunnelObject
        }

        var tunnelStatus: TunnelStatus {
            get {
                nslock.lock()
                defer { nslock.unlock() }

                return _tunnelStatus
            }
            set {
                nslock.lock()
                defer { nslock.unlock() }

                guard _tunnelStatus != newValue else { return }

                _tunnelStatus = newValue

                delegateQueue.async {
                    self.delegate?.tunnelManagerState(self, didChangeTunnelStatus: newValue)
                }
            }
        }

        init(delegateQueue: DispatchQueue) {
            self.delegateQueue = delegateQueue
        }

        func setTunnel(_ newTunnelObject: Tunnel?, shouldRefreshTunnelState: Bool) {
            nslock.lock()
            defer { nslock.unlock() }

            guard _tunnelObject != newTunnelObject else { return }

            _tunnelObject = newTunnelObject

            delegateQueue.async {
                self.delegate?.tunnelManagerState(
                    self,
                    didChangeTunnelProvider: newTunnelObject,
                    shouldRefreshTunnelState: shouldRefreshTunnelState
                )
            }
        }
    }
}
