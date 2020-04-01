//
//  SimulatorTunnelProviderHost.swift
//  MullvadVPN
//
//  Created by pronebird on 10/02/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

#if targetEnvironment(simulator)

import Combine
import Foundation
import Network
import NetworkExtension

class SimulatorTunnelProviderHost: SimulatorTunnelProviderDelegate {

    private let cancellableSet = CancellableSet()
    private var connectionInfo: TunnelConnectionInfo?

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
        PacketTunnelIpcHandler.decodeRequest(messageData: messageData)
            .receive(on: DispatchQueue.main)
            .flatMap { (request) -> AnyPublisher<AnyEncodable, PacketTunnelIpcHandlerError> in
                switch request {
                case .reloadConfiguration:
                    return Result.Publisher(AnyEncodable(true))
                        .eraseToAnyPublisher()

                case .tunnelInformation:
                    return Result.Publisher(AnyEncodable(self.connectionInfo))
                        .eraseToAnyPublisher()
                }
        }.flatMap({ (response) in
            return PacketTunnelIpcHandler.encodeResponse(response: response)
        }).autoDisposableSink(cancellableSet: cancellableSet, receiveCompletion: { (completion) in
            if case .failure = completion {
                completionHandler?(nil)
            }
        }, receiveValue: { (responseData) in
            completionHandler?(responseData)
        })
    }

}

#endif
