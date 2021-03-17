//
//  SimulatorTunnelProviderHost.swift
//  MullvadVPN
//
//  Created by pronebird on 10/02/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

#if targetEnvironment(simulator)

import Foundation
import Network
import NetworkExtension
import Logging

class SimulatorTunnelProviderHost: SimulatorTunnelProviderDelegate {

    private enum ExclusivityCategory {
        case exclusive
    }

    private var connectionInfo: TunnelConnectionInfo?
    private let logger = Logger(label: "SimulatorTunnelProviderHost")

    private let operationQueue = OperationQueue()
    private lazy var exclusivityController = ExclusivityController<ExclusivityCategory>(operationQueue: operationQueue)

    override func startTunnel(options: [String: Any]?, completionHandler: @escaping (Error?) -> Void) {
        let startOperation = makeStartOperation()
        startOperation.addDidFinishBlockObserver(queue: .main) { _ in
            completionHandler(nil)
        }
        exclusivityController.addOperation(startOperation, categories: [.exclusive])
    }

    override func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        DispatchQueue.main.async {
            self.connectionInfo = nil

            completionHandler()
        }
    }

    override func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)?) {
        DispatchQueue.main.async {
            let result = PacketTunnelIpcHandler.decodeRequest(messageData: messageData)
            switch result {
            case .success(let request):
                switch request {
                case .reloadTunnelSettings:
                    let operationObserver = OperationBlockObserver<AsyncBlockOperation>(
                        queue: .main,
                        willExecute: { _ in
                            self.reasserting = true
                        },
                        willFinish: { _ in
                            self.reasserting = false
                        },
                        didFinish: { _ in
                            self.replyAppMessage(true, completionHandler: completionHandler)
                        })

                    let startOperation = self.makeStartOperation()
                    startOperation.addObserver(operationObserver)
                    self.exclusivityController.addOperation(startOperation, categories: [.exclusive])

                case .tunnelInformation:
                    self.replyAppMessage(self.connectionInfo, completionHandler: completionHandler)
                }

            case .failure:
                completionHandler?(nil)
            }
        }
    }

    private func replyAppMessage<T: Codable>(_ response: T, completionHandler: ((Data?) -> Void)?) {
        switch PacketTunnelIpcHandler.encodeResponse(response: response) {
        case .success(let data):
            completionHandler?(data)

        case .failure(let error):
            self.logger.error(chainedError: error)
            completionHandler?(nil)
        }
    }

    private func makeStartOperation() -> AsyncBlockOperation {
        return AsyncBlockOperation { [weak self] (finish) in
            guard let self = self else {
                finish()
                return
            }

            self.pickRelay { (selectorResult) in
                DispatchQueue.main.async {
                    self.connectionInfo = selectorResult?.tunnelConnectionInfo
                    finish()
                }
            }
        }
    }

    private func pickRelay(completion: @escaping (RelaySelectorResult?) -> Void) {
        RelayCache.shared.read { (result) in
            switch result {
            case .success(let cachedRelays):
                let keychainReference = self.protocolConfiguration.passwordReference!
                switch TunnelSettingsManager.load(searchTerm: .persistentReference(keychainReference)) {
                case .success(let entry):
                    let relayConstraints = entry.tunnelSettings.relayConstraints
                    let relaySelector = RelaySelector(relays: cachedRelays.relays)
                    let selectorResult = relaySelector.evaluate(with: relayConstraints)
                    completion(selectorResult)

                case .failure(let error):
                    self.logger.error(chainedError: error)
                    completion(nil)
                }
            case .failure(let error):
                self.logger.error(chainedError: error)
                completion(nil)
            }
        }
    }

}

#endif
