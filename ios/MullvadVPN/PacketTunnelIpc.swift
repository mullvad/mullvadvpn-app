//
//  PacketTunnelIpc.swift
//  MullvadVPN
//
//  Created by pronebird on 01/11/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Combine
import Foundation
import NetworkExtension

/// A enum describing the kinds of requests that `PacketTunnelProvider` handles
enum PacketTunnelRequest: Int, Codable {
    /// Request the tunnel to reload the configuration
    case reloadConfiguration

    /// Request the tunnel to return the connection information
    case tunnelInformation
}

enum PacketTunnelIpcError: Error {
    /// A failure to encode the request
    case encoding(Error)

    /// A failure to decode the response
    case decoding(Error)

    /// A failure to send the IPC request
    case send(Error)

    /// A failure that's raised when the IPC response does not contain any data however the decoder
    /// expected to receive data for decoding
    case nilResponse

    var localizedDescription: String {
        switch self {
        case .encoding(let error):
            return "Encoding failure: \(error.localizedDescription)"
        case .decoding(let error):
            return "Decoding failure: \(error.localizedDescription)"
        case .send(let error):
            return "Submission failure: \(error.localizedDescription)"
        case .nilResponse:
            return "Unexpected nil response from the tunnel"
        }
    }
}

/// A struct that holds the basic information regarding the tunnel connection
struct TunnelConnectionInfo: Codable, Equatable {
    let ipv4Relay: String
    let ipv6Relay: String?
    let hostname: String
}

extension TunnelConnectionInfo: CustomDebugStringConvertible {
    var debugDescription: String {
        return "{ ipv4Relay: \(String(reflecting: ipv4Relay)), " +
               "ipv6Relay: \(String(reflecting: ipv6Relay)), " +
               "hostname: \(String(reflecting: hostname)) }"
    }
}

enum PacketTunnelIpcHandlerError: Error {
    /// A failure to encode the request
    case encoding(Error)

    /// A failure to decode the response
    case decoding(Error)

    /// A failure to process the request
    case processing(Error)
}

enum PacketTunnelIpcHandler {}

extension PacketTunnelIpcHandler {

    static func decodeRequest(messageData: Data) -> AnyPublisher<PacketTunnelRequest, PacketTunnelIpcHandlerError> {
        return Just(messageData)
            .setFailureType(to: PacketTunnelIpcHandlerError.self)
            .decode(type: PacketTunnelRequest.self, decoder: JSONDecoder())
            .mapError { PacketTunnelIpcHandlerError.decoding($0) }
            .eraseToAnyPublisher()
    }

    static func encodeResponse<T>(response: T) -> AnyPublisher<Data, PacketTunnelIpcHandlerError> where T: Encodable {
        return Just(response)
            .setFailureType(to: PacketTunnelIpcHandlerError.self)
            .encode(encoder: JSONEncoder())
            .mapError { PacketTunnelIpcHandlerError.encoding($0) }
            .eraseToAnyPublisher()
    }
}

class PacketTunnelIpc {
    let session: NETunnelProviderSession

    init(session: NETunnelProviderSession) {
        self.session = session
    }

    func reloadConfiguration() -> AnyPublisher<(), PacketTunnelIpcError> {
        return send(message: .reloadConfiguration)
    }

    func getTunnelInformation() -> AnyPublisher<TunnelConnectionInfo, PacketTunnelIpcError> {
        return send(message: .tunnelInformation)
    }

    private func send(message: PacketTunnelRequest) -> AnyPublisher<(), PacketTunnelIpcError> {
        return sendWithoutDecoding(message: message)
            .map { _ in () }.eraseToAnyPublisher()
    }

    private func send<T>(message: PacketTunnelRequest) -> AnyPublisher<T, PacketTunnelIpcError> where T: Decodable {
        return sendWithoutDecoding(message: message)
            .replaceNil(with: .nilResponse)
            .decode(type: T.self, decoder: JSONDecoder())
            .mapError { PacketTunnelIpcError.decoding($0) }
            .eraseToAnyPublisher()
    }

    private func sendWithoutDecoding(message: PacketTunnelRequest) -> AnyPublisher<Data?, PacketTunnelIpcError> {
        return Just(message)
            .setFailureType(to: PacketTunnelIpcError.self)
            .encode(encoder: JSONEncoder())
            .mapError { PacketTunnelIpcError.encoding($0) }
            .flatMap(self.sendProviderMessage)
            .mapError { PacketTunnelIpcError.send($0) }
            .eraseToAnyPublisher()
    }

    private func sendProviderMessage(_ messageData: Data) -> Future<Data?, Error> {
        return Future { (fulfill) in
            do {
                try self.session.sendProviderMessage(messageData, responseHandler: { (response) in
                    fulfill(.success(response))
                })
            } catch {
                fulfill(.failure(error))
            }
        }
    }

}
