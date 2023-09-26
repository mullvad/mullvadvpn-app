//
//  AppMessageHandlerTests.swift
//  PacketTunnelCoreTests
//
//  Created by Jon Petersson on 2023-09-28.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
@testable import MullvadREST
import MullvadTypes
import PacketTunnelCore
import RelaySelector
import WireGuardKitTypes
import XCTest

final class AppMessageHandlerTests: XCTestCase {
    private var stateSink: Combine.Cancellable?

    private lazy var urlRequestProxy: URLRequestProxy = {
        let transportProvider = REST.AnyTransportProvider {
            AnyTransport { Response(delay: 1, statusCode: 200, value: TimeResponse(dateTime: Date())) }
        }

        return URLRequestProxy(
            dispatchQueue: DispatchQueue(label: "AppMessageHandlerTests"),
            transportProvider: transportProvider
        )
    }()

    override func tearDown() async throws {
        stateSink?.cancel()
    }

    func testHandleAppMessageForTunnelStatus() async throws {
        let actor = PacketTunnelActor.mock()
        let appMessageHandler = createAppMessageHandler(actor: actor)
        let reconnectingExpectation = expectation(description: "Expect reconnecting state")

        actor.start(options: StartOptions(launchSource: .app))
        actor.setErrorState(reason: .deviceRevoked)

        stateSink = await actor.$state
            .receive(on: DispatchQueue.main)
            .sink { newState in
                switch newState {
                case .error:
                    Task {
                        let reply = try? await appMessageHandler.handleAppMessage(
                            TunnelProviderMessage.getTunnelStatus.encode()
                        )
                        let tunnelStatus = try? TunnelProviderReply<PacketTunnelStatus>(messageData: reply!)

                        XCTAssertEqual(tunnelStatus?.value.blockedStateReason, .deviceRevoked)
                        reconnectingExpectation.fulfill()
                    }
                default:
                    break
                }
            }

        await fulfillment(of: [reconnectingExpectation], timeout: 1)
    }

    func testHandleAppMessageForURLRequest() async throws {
        let actor = PacketTunnelActor.mock()
        let appMessageHandler = createAppMessageHandler(actor: actor)

        actor.start(options: StartOptions(launchSource: .app))

        let url = URL(string: "localhost")!
        let urlRequest = ProxyURLRequest(
            id: UUID(),
            urlRequest: URLRequest(url: url)
        )!

        let reply = try await appMessageHandler
            .handleAppMessage(TunnelProviderMessage.sendURLRequest(urlRequest).encode())
        let urlResponse = try TunnelProviderReply<ProxyURLResponse>(messageData: reply!)

        XCTAssertEqual(urlResponse.value.response?.url, url)
    }

    func testHandleAppMessageForReconnectTunnel() async throws {
        let actor = PacketTunnelActor.mock()
        let appMessageHandler = createAppMessageHandler(actor: actor)
        let reconnectingExpectation = expectation(description: "Expect reconnecting state")

        actor.start(options: StartOptions(launchSource: .app))

        let relayConstraints = RelayConstraints(location: .only(.hostname("se", "sto", "se6-wireguard")))
        let selectorResult = try? RelaySelector.evaluate(
            relays: sampleRelays,
            constraints: relayConstraints,
            numberOfFailedAttempts: 0
        )

        _ = try? await appMessageHandler.handleAppMessage(
            TunnelProviderMessage.reconnectTunnel(selectorResult).encode()
        )

        stateSink = await actor.$state
            .receive(on: DispatchQueue.main)
            .sink { newState in
                switch newState {
                case let .reconnecting(state):
                    XCTAssertEqual(state.selectedRelay.relay, selectorResult?.relay)
                    reconnectingExpectation.fulfill()
                default:
                    break
                }
            }

        await fulfillment(of: [reconnectingExpectation], timeout: 1)
    }

    func testHandleAppMessageForKeyRotation() async throws {
        let actor = PacketTunnelActor.mock()
        let appMessageHandler = createAppMessageHandler(actor: actor)
        let connectedExpectation = expectation(description: "Expect connecting state")

        actor.start(options: StartOptions(launchSource: .app))

        _ = try? await appMessageHandler.handleAppMessage(
            TunnelProviderMessage.privateKeyRotation.encode()
        )

        stateSink = await actor.$state
            .receive(on: DispatchQueue.main)
            .sink { newState in
                switch newState {
                case let .connected(state):
                    XCTAssertNotNil(state.lastKeyRotation)
                    connectedExpectation.fulfill()
                default:
                    break
                }
            }

        await fulfillment(of: [connectedExpectation], timeout: 3)
    }
}

extension AppMessageHandlerTests {
    func createAppMessageHandler(actor: PacketTunnelActor) -> AppMessageHandler {
        return AppMessageHandler(
            packetTunnelActor: actor,
            urlRequestProxy: urlRequestProxy
        )
    }
}

struct TimeResponse: Codable {
    var dateTime: Date
}

class AnyTransport: RESTTransport {
    typealias CompletionHandler = (Data?, URLResponse?, Error?) -> Void

    private let handleRequest: () -> AnyResponse

    private let completionLock = NSLock()
    private var completionHandlers: [UUID: CompletionHandler] = [:]

    init(block: @escaping () -> AnyResponse) {
        handleRequest = block
    }

    var name: String {
        return "any-transport"
    }

    func sendRequest(
        _ request: URLRequest,
        completion: @escaping (Data?, URLResponse?, Error?) -> Void
    ) -> MullvadTypes.Cancellable {
        let response = handleRequest()
        let id = storeCompletion(completionHandler: completion)

        let dispatchWork = DispatchWorkItem {
            let data = (try? response.encode()) ?? Data()
            let httpResponse = HTTPURLResponse(
                url: request.url!,
                statusCode: response.statusCode,
                httpVersion: "1.0",
                headerFields: [:]
            )!
            self.sendCompletion(requestID: id, completion: .success((data, httpResponse)))
        }

        DispatchQueue.global().asyncAfter(deadline: .now() + response.delay, execute: dispatchWork)

        return AnyCancellable {
            dispatchWork.cancel()

            self.sendCompletion(requestID: id, completion: .failure(URLError(.cancelled)))
        }
    }

    private func storeCompletion(completionHandler: @escaping CompletionHandler) -> UUID {
        return completionLock.withLock {
            let id = UUID()
            completionHandlers[id] = completionHandler
            return id
        }
    }

    private func sendCompletion(requestID: UUID, completion: Result<(Data, URLResponse), Error>) {
        let complationHandler = completionLock.withLock {
            return completionHandlers.removeValue(forKey: requestID)
        }
        switch completion {
        case let .success((data, response)):
            complationHandler?(data, response, nil)
        case let .failure(error):
            complationHandler?(nil, nil, error)
        }
    }
}

struct Response<T: Encodable>: AnyResponse {
    var delay: TimeInterval
    var statusCode: Int
    var value: T

    func encode() throws -> Data {
        return try REST.Coding.makeJSONEncoder().encode(value)
    }
}

protocol AnyResponse {
    var delay: TimeInterval { get }
    var statusCode: Int { get }

    func encode() throws -> Data
}

private let portRanges: [[UInt16]] = [[4000, 4001], [5000, 5001]]

private let sampleRelays = REST.ServerRelaysResponse(
    locations: [
        "es-mad": REST.ServerLocation(
            country: "Spain",
            city: "Madrid",
            latitude: 40.408566,
            longitude: -3.69222
        ),
        "se-got": REST.ServerLocation(
            country: "Sweden",
            city: "Gothenburg",
            latitude: 57.70887,
            longitude: 11.97456
        ),
        "se-sto": REST.ServerLocation(
            country: "Sweden",
            city: "Stockholm",
            latitude: 59.3289,
            longitude: 18.0649
        ),
        "ae-dxb": REST.ServerLocation(
            country: "United Arab Emirates",
            city: "Dubai",
            latitude: 25.276987,
            longitude: 55.296249
        ),
        "jp-tyo": REST.ServerLocation(
            country: "Japan",
            city: "Tokyo",
            latitude: 35.685,
            longitude: 139.751389
        ),
        "ca-tor": REST.ServerLocation(
            country: "Canada",
            city: "Toronto",
            latitude: 43.666667,
            longitude: -79.416667
        ),
        "us-atl": REST.ServerLocation(
            country: "USA",
            city: "Atlanta, GA",
            latitude: 40.73061,
            longitude: -73.935242
        ),
        "us-dal": REST.ServerLocation(
            country: "USA",
            city: "Dallas, TX",
            latitude: 32.89748,
            longitude: -97.040443
        ),
    ],
    wireguard: REST.ServerWireguardTunnels(
        ipv4Gateway: .loopback,
        ipv6Gateway: .loopback,
        portRanges: portRanges,
        relays: [
            REST.ServerRelay(
                hostname: "es1-wireguard",
                active: true,
                owned: true,
                location: "es-mad",
                provider: "",
                weight: 500,
                ipv4AddrIn: .loopback,
                ipv6AddrIn: .loopback,
                publicKey: PrivateKey().publicKey.rawValue,
                includeInCountry: true
            ),
            REST.ServerRelay(
                hostname: "se10-wireguard",
                active: true,
                owned: true,
                location: "se-got",
                provider: "",
                weight: 1000,
                ipv4AddrIn: .loopback,
                ipv6AddrIn: .loopback,
                publicKey: PrivateKey().publicKey.rawValue,
                includeInCountry: true
            ),
            REST.ServerRelay(
                hostname: "se2-wireguard",
                active: true,
                owned: true,
                location: "se-sto",
                provider: "",
                weight: 50,
                ipv4AddrIn: .loopback,
                ipv6AddrIn: .loopback,
                publicKey: PrivateKey().publicKey.rawValue,
                includeInCountry: true
            ),
            REST.ServerRelay(
                hostname: "se6-wireguard",
                active: true,
                owned: true,
                location: "se-sto",
                provider: "",
                weight: 100,
                ipv4AddrIn: .loopback,
                ipv6AddrIn: .loopback,
                publicKey: PrivateKey().publicKey.rawValue,
                includeInCountry: true
            ),
            REST.ServerRelay(
                hostname: "us-dal-wg-001",
                active: true,
                owned: true,
                location: "us-dal",
                provider: "",
                weight: 100,
                ipv4AddrIn: .loopback,
                ipv6AddrIn: .loopback,
                publicKey: PrivateKey().publicKey.rawValue,
                includeInCountry: true
            ),
            REST.ServerRelay(
                hostname: "us-nyc-wg-301",
                active: true,
                owned: true,
                location: "us-nyc",
                provider: "",
                weight: 100,
                ipv4AddrIn: .loopback,
                ipv6AddrIn: .loopback,
                publicKey: PrivateKey().publicKey.rawValue,
                includeInCountry: true
            ),
        ]
    ),
    bridge: REST.ServerBridges(shadowsocks: [
        REST.ServerShadowsocks(protocol: "tcp", port: 443, cipher: "aes-256-gcm", password: "mullvad"),
    ], relays: [
        REST.BridgeRelay(
            hostname: "se-sto-br-001",
            active: true,
            owned: true,
            location: "se-sto",
            provider: "31173",
            ipv4AddrIn: .loopback,
            weight: 100,
            includeInCountry: true
        ),
        REST.BridgeRelay(
            hostname: "jp-tyo-br-101",
            active: true,
            owned: true,
            location: "jp-tyo",
            provider: "M247",
            ipv4AddrIn: .loopback,
            weight: 1,
            includeInCountry: true
        ),
        REST.BridgeRelay(
            hostname: "ca-tor-ovpn-001",
            active: false,
            owned: false,
            location: "ca-tor",
            provider: "M247",
            ipv4AddrIn: .loopback,
            weight: 1,
            includeInCountry: true
        ),
        REST.BridgeRelay(
            hostname: "ae-dxb-ovpn-001",
            active: true,
            owned: false,
            location: "ae-dxb",
            provider: "M247",
            ipv4AddrIn: .loopback,
            weight: 100,
            includeInCountry: true
        ),
        REST.BridgeRelay(
            hostname: "us-atl-br-101",
            active: true,
            owned: false,
            location: "us-atl",
            provider: "100TB",
            ipv4AddrIn: .loopback,
            weight: 100,
            includeInCountry: true
        ),
        REST.BridgeRelay(
            hostname: "us-dal-br-101",
            active: true,
            owned: false,
            location: "us-dal",
            provider: "100TB",
            ipv4AddrIn: .loopback,
            weight: 100,
            includeInCountry: true
        ),
    ])
)
