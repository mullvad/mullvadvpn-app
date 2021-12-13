//
//  TunnelIPCSession.swift
//  MullvadVPN
//
//  Created by pronebird on 16/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension TunnelIPC {
    /// Wrapper class around `NETunnelProviderSession` that provides convenient interface for interacting with the
    /// Packet Tunnel process.
    final class Session {
        private let tunnelProviderSession: VPNTunnelProviderSessionProtocol

        init<T: VPNTunnelProviderManagerProtocol>(from tunnelProvider: T) {
            tunnelProviderSession = tunnelProvider.connection as! VPNTunnelProviderSessionProtocol
        }

        func reloadTunnelSettings(completionHandler: @escaping (TunnelIPC.Error?) -> Void) {
            send(message: .reloadTunnelSettings) { result in
                completionHandler(result.error)
            }
        }

        func getTunnelConnectionInfo(completionHandler: @escaping (Result<TunnelConnectionInfo?, Error>) -> Void) {
            send(message: .tunnelConnectionInfo) { result in
                completionHandler(result)
            }
        }

        // MARK: - Private

        private func send(message: TunnelIPC.Request, completionHandler: @escaping (Result<(), Error>) -> Void) {
            sendWithoutDecoding(message: message) { (result) in
                let result = result.map { _ in () }

                completionHandler(result)
            }
        }

        private func send<T>(message: TunnelIPC.Request, completionHandler: @escaping (Result<T, Error>) -> Void) where T: Codable
        {
            sendWithoutDecoding(message: message) { (result) in
                let result = result.flatMap { (data) -> Result<T, Error> in
                    guard let data = data else {
                        return .failure(.nilResponse)
                    }

                    return Result { try TunnelIPC.Coding.decodeResponse(T.self, from: data) }
                        .mapError { error in
                            return .decoding(error)
                        }
                }

                completionHandler(result)
            }
        }

        private func sendWithoutDecoding(message: TunnelIPC.Request, completionHandler: @escaping (Result<Data?, Error>) -> Void) {
            do {
                let data = try TunnelIPC.Coding.encodeRequest(message)

                sendProviderMessage(data) { (result) in
                    completionHandler(result)
                }
            } catch {
                completionHandler(.failure(.encoding(error)))
            }
        }

        private func sendProviderMessage(_ messageData: Data, completionHandler: @escaping (Result<Data?, Error>) -> Void) {
            do {
                try tunnelProviderSession.sendProviderMessage(messageData) { response in
                    completionHandler(.success(response))
                }
            } catch {
                completionHandler(.failure(.send(error)))
            }
        }

    }
}
