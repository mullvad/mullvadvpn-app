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

    private var connectionInfo: TunnelConnectionInfo?
    private let logger = Logger(label: "SimulatorTunnelProviderHost")

    func startTunnel(options: [String: Any]?, completionHandler: @escaping (Error?) -> Void) {
        DispatchQueue.main.async {
            self.connectionInfo = TunnelConnectionInfo(
                ipv4Relay: IPv4Endpoint(ip: IPv4Address("10.0.0.1")!, port: 53),
                ipv6Relay: nil,
                hostname: "au4-wireguard",
                location: Location(
                    country: "Australia",
                    countryCode: "au",
                    city: "Melbourne",
                    cityCode: "mel",
                    latitude: -37.815018,
                    longitude: 144.946014
                )
            )

            completionHandler(nil)
        }
    }

    func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        DispatchQueue.main.async {
            self.connectionInfo = nil

            completionHandler()
        }
    }

    func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)?) {
        DispatchQueue.main.async {
            let result = PacketTunnelIpcHandler.decodeRequest(messageData: messageData)
            switch result {
            case .success(let request):
                switch request {
                case .reloadTunnelSettings:
                    self.replyAppMessage(true, completionHandler: completionHandler)

                case .tunnelInformation:
                    self.replyAppMessage(self.connectionInfo, completionHandler: completionHandler)
                }

            case .failure:
                completionHandler?(nil)
            }
        }
    }

    private func replyAppMessage<T: Encodable>(_ response: T, completionHandler: ((Data?) -> Void)?)
    {
        switch PacketTunnelIpcHandler.encodeResponse(response: response) {
        case .success(let data):
            completionHandler?(data)

        case .failure(let error):
            self.logger.error(chainedError: error)
            completionHandler?(nil)
        }
    }

}

#endif
