//
//  PacketTunnelIpc.swift
//  MullvadVPN
//
//  Created by pronebird on 01/11/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

/// A enum describing the kinds of requests that `PacketTunnelProvider` handles
enum PacketTunnelRequest: Int, Codable, RawRepresentable {
    /// Request the tunnel to reload settings
    case reloadTunnelSettings

    /// Request the tunnel to return the connection information
    case tunnelInformation

    private enum CodingKeys: String, CodingKey {
        case type
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(rawValue, forKey: CodingKeys.type)
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let rawValue = try container.decode(RawValue.self, forKey: CodingKeys.type)

        if let decoded = PacketTunnelRequest(rawValue: rawValue) {
            self = decoded
        } else {
            throw DecodingError.dataCorruptedError(
                forKey: CodingKeys.type,
                in: container,
                debugDescription: "Unrecognized raw value."
            )
        }
    }
}

/// A struct that holds the basic information regarding the tunnel connection
struct TunnelConnectionInfo: Codable, Equatable {
    let ipv4Relay: IPv4Endpoint
    let ipv6Relay: IPv6Endpoint?
    let hostname: String
    let location: Location
}

extension TunnelConnectionInfo: CustomDebugStringConvertible {
    var debugDescription: String {
        return "{ ipv4Relay: \(String(reflecting: ipv4Relay)), " +
               "ipv6Relay: \(String(reflecting: ipv6Relay)), " +
               "hostname: \(String(reflecting: hostname))," +
               "location: \(String(reflecting: location)) }"
    }
}

enum PacketTunnelIpcHandler {}

extension PacketTunnelIpcHandler {

    enum Error: ChainedError {
        /// A failure to encode the request
        case encoding(Swift.Error)

        /// A failure to decode the response
        case decoding(Swift.Error)

        /// A failure to process the request
        case processing(Swift.Error)

        var errorDescription: String? {
            switch self {
            case .encoding:
                return "Encoding failure"
            case .decoding:
                return "Decoding failure"
            case .processing:
                return "Request handling failure"
            }
        }
    }

    static func decodeRequest(messageData: Data) -> Result<PacketTunnelRequest, Error> {
        do {
            let decoder = JSONDecoder()
            let value = try decoder.decode(PacketTunnelRequest.self, from: messageData)

            return .success(value)
        } catch {
            return .failure(.decoding(error))
        }
    }

    static func encodeResponse<T>(response: T) -> Result<Data, Error> where T: Encodable {
        do {
            let encoder = JSONEncoder()
            let value = try encoder.encode(response)

            return .success(value)
        } catch {
            return .failure(.encoding(error))
        }
    }
}

class PacketTunnelIpc {

    enum Error: ChainedError {
        /// A failure to encode the request
        case encoding(Swift.Error)

        /// A failure to decode the response
        case decoding(Swift.Error)

        /// A failure to send the IPC request
        case send(Swift.Error)

        /// A failure that's raised when the IPC response does not contain any data however the decoder
        /// expected to receive data for decoding
        case nilResponse

        var errorDescription: String? {
            switch self {
            case .encoding:
                return "Encoding failure"
            case .decoding:
                return "Decoding failure"
            case .send:
                return "Submission failure"
            case .nilResponse:
                return "Unexpected nil response from the tunnel"
            }
        }
    }

    let session: VPNTunnelProviderSessionProtocol

    init(session: VPNTunnelProviderSessionProtocol) {
        self.session = session
    }

    func reloadTunnelSettings(completionHandler: @escaping (Result<(), Error>) -> Void) {
        send(message: .reloadTunnelSettings, completionHandler: completionHandler)
    }

    func getTunnelInformation(completionHandler: @escaping (Result<TunnelConnectionInfo, Error>) -> Void) {
        send(message: .tunnelInformation, completionHandler: completionHandler)
    }

    private class func encodeRequest(message: PacketTunnelRequest) -> Result<Data, Error> {
        do {
            let encoder = JSONEncoder()
            let data = try encoder.encode(message)

            return .success(data)
        } catch {
            return .failure(.encoding(error))
        }
    }

    private class func decodeResponse<T>(data: Data) -> Result<T, Error> where T: Decodable {
        do {
            let decoder = JSONDecoder()
            let value = try decoder.decode(T.self, from: data)

            return .success(value)
        } catch {
            return .failure(.decoding(error))
        }
    }

    private func send(message: PacketTunnelRequest, completionHandler: @escaping (Result<(), Error>) -> Void) {
        sendWithoutDecoding(message: message) { (result) in
            let result = result.map { _ in () }

            completionHandler(result)
        }
    }

    private func send<T>(message: PacketTunnelRequest, completionHandler: @escaping (Result<T, Error>) -> Void)
        where T: Decodable
    {
        sendWithoutDecoding(message: message) { (result) in
            let result = result.flatMap { (data) -> Result<T, Error> in
                if let data = data {
                    return Self.decodeResponse(data: data)
                } else {
                    return .failure(.nilResponse)
                }
            }

            completionHandler(result)
        }
    }

    private func sendWithoutDecoding(message: PacketTunnelRequest, completionHandler: @escaping (Result<Data?, Error>) -> Void) {
        switch Self.encodeRequest(message: message) {
        case .success(let data):
            self.sendProviderMessage(data) { (result) in
                completionHandler(result)
            }

        case .failure(let error):
            completionHandler(.failure(error))
        }
    }

    private func sendProviderMessage(_ messageData: Data, completionHandler: @escaping (Result<Data?, Error>) -> Void) {
        do {
            try self.session.sendProviderMessage(messageData, responseHandler: { (response) in
                completionHandler(.success(response))
            })
        } catch {
            completionHandler(.failure(.send(error)))
        }
    }

}
