//
//  PacketCaptureAPIClient.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-04-30.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

struct PacketCaptureSession {
    var identifier = UUID().uuidString
}

/// Represents a stream in packet capture
class Stream: Codable, Equatable {
    static func == (lhs: Stream, rhs: Stream) -> Bool {
        return lhs.sourceAddress == rhs.sourceAddress &&
            lhs.destinationAddress == rhs.destinationAddress &&
            lhs.flowID == rhs.flowID &&
            lhs.transportProtocol == rhs.transportProtocol
    }

    let sourceAddress: String
    let sourcePort: Int
    let destinationAddress: String
    let destinationPort: Int
    let flowID: String?
    let transportProtocol: NetworkTransportProtocol
    var packets: [Packet]

    /// Date interval from first to last packet of this stream
    var dateInterval: DateInterval

    /// Date interval from first to last tx(sent from test device) packet of this stream
    var txInterval: DateInterval?

    /// Date interval from frist to last rx(sent to test device) packet of this stream
    var rxInterval: DateInterval?

    enum CodingKeys: String, CodingKey {
        case sourceAddress = "peer_addr"
        case destinationAddress = "other_addr"
        case flowID = "flow_id"
        case transportProtocol = "transport_protocol"
        case packets
    }

    required init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        self.flowID = try container.decodeIfPresent(String.self, forKey: .flowID)
        self.transportProtocol = try container.decode(NetworkTransportProtocol.self, forKey: .transportProtocol)
        self.packets = try container.decode([Packet].self, forKey: .packets)
        dateInterval = DateInterval()

        // Separate source address and port
        let sourceValue = try container.decode(String.self, forKey: .sourceAddress)
        let sourceSplit = sourceValue.components(separatedBy: ":")
        self.sourceAddress = try XCTUnwrap(sourceSplit.first)
        self.sourcePort = try XCTUnwrap(Int(try XCTUnwrap(sourceSplit.last)))

        // Separate destination address and port
        let destinationValue = try container.decode(String.self, forKey: .destinationAddress)
        let destinationSplit = destinationValue.components(separatedBy: ":")
        self.destinationAddress = try XCTUnwrap(destinationSplit.first)
        self.destinationPort = try XCTUnwrap(Int(try XCTUnwrap(destinationSplit.last)))

        // Set date interval based on packets' time window
        determineDateInterval()
    }

    /// Determine the stream's date interval from the time between first to the last packet
    private func determineDateInterval() {
        guard let firstPackageDate = packets.first?.date else {
            XCTFail("Stream unexpectedly have no packets")
            return
        }

        // Identify first tx and rx packets to set as initial values
        let txPackets = packets.filter { $0.fromPeer == true }.sorted { $0.date < $1.date }
        let rxPackets = packets.filter { $0.fromPeer == false }.sorted { $0.date < $1.date }
        let allPackets = packets.sorted { $0.date < $1.date }

        if let firstTxPacket = txPackets.first, let lastTxPacket = txPackets.last {
            txInterval = DateInterval(start: firstTxPacket.date, end: lastTxPacket.date)
        }

        if let firstRxPacket = rxPackets.first, let lastRxPacket = rxPackets.last {
            rxInterval = DateInterval(start: firstRxPacket.date, end: lastRxPacket.date)
        }

        if let firstPacket = allPackets.first, let lastPacket = allPackets.last {
            dateInterval = DateInterval(start: firstPacket.date, end: lastPacket.date)
        }
    }

    public func removePacket(packet: Packet) {
        guard let packetIndex = packets.firstIndex(of: packet) else {
            XCTFail("Get packet index")
            return
        }

        packets.remove(at: packetIndex)
    }
}

/// Represents a packet in packet capture
class Packet: Codable, Equatable {
    /// True when packet is sent from device under test, false if from another host
    public let fromPeer: Bool

    /// Timestamp in microseconds
    private var timestamp: Int64

    public var date: Date

    enum CodingKeys: String, CodingKey {
        case fromPeer = "from_peer"
        case timestamp
    }

    required init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        fromPeer = try container.decode(Bool.self, forKey: .fromPeer)
        timestamp = try container.decode(Int64.self, forKey: .timestamp) / 1000000
        date = Date(timeIntervalSince1970: TimeInterval(timestamp))
    }

    static func == (lhs: Packet, rhs: Packet) -> Bool {
        return lhs.fromPeer == rhs.fromPeer &&
            lhs.timestamp == rhs.timestamp &&
            lhs.date == rhs.date
    }
}

class PacketCapture {
    // swiftlint:disable force_cast
    let baseURL = URL(
        string: Bundle(for: PacketCapture.self)
            .infoDictionary?["PacketCaptureAPIBaseURL"] as! String
    )!
    // swiftlint:enable force_cast

    /// Start a new capture session
    func startCapture() -> PacketCaptureSession {
        let session = PacketCaptureSession()

        let jsonDictionary = [
            "label": session.identifier,
        ]

        _ = sendRequest(
            httpMethod: "POST",
            endpoint: "capture",
            contentType: "application/json",
            jsonData: jsonDictionary
        )

        return session
    }

    /// Stop capture for session
    func stopCapture(session: PacketCaptureSession) {
        _ = sendJSONRequest(httpMethod: "POST", endpoint: "stop-capture/\(session.identifier)", jsonData: nil)
    }

    /// Get captured traffic from this session parsed to objects
    func getParsedCaptureObjects(session: PacketCaptureSession) -> [Stream] {
        let parsedData = getParsedCapture(session: session)
        let decoder = JSONDecoder()

        do {
            let streams = try decoder.decode([Stream].self, from: parsedData)
            return streams
        } catch {
            XCTFail("Failed to decode parsed capture")
            return []
        }
    }

    /// Get captured traffic from this session parsed to JSON
    func getParsedCapture(session: PacketCaptureSession) -> Data {
        var deviceIPAddress: String

        do {
            deviceIPAddress = try Networking.getIPAddress()
        } catch {
            XCTFail("Failed to get device IP address")
            return Data()
        }

        let responseData = sendJSONRequest(
            httpMethod: "PUT",
            endpoint: "parse-capture/\(session.identifier)",
            jsonData: [deviceIPAddress]
        )

        return responseData
    }

    /// Get PCAP file contents for the capture of this session
    func getPCAP(session: PacketCaptureSession) -> Data {
        let response = sendPCAPRequest(httpMethod: "GET", endpoint: "last-capture/\(session.identifier)", jsonData: nil)
        return response
    }

    private func sendJSONRequest(httpMethod: String, endpoint: String, jsonData: Any?) -> Data {
        let responseData = sendRequest(
            httpMethod: httpMethod,
            endpoint: endpoint,
            contentType: "application/json",
            jsonData: jsonData
        )

        guard let responseData else {
            XCTFail("Unexpectedly didn't get any data from JSON request")
            return Data()
        }

        return responseData
    }

    private func sendPCAPRequest(httpMethod: String, endpoint: String, jsonData: Any?) -> Data {
        let responseData = sendRequest(
            httpMethod: httpMethod,
            endpoint: endpoint,
            contentType: "application/pcap",
            jsonData: jsonData
        )

        guard let responseData else {
            XCTFail("Unexpectedly didn't get any data from response")
            return Data()
        }

        XCTAssertFalse(responseData.isEmpty, "PCAP response data should not be empty")

        return responseData
    }

    private func sendRequest(httpMethod: String, endpoint: String, contentType: String?, jsonData: Any?) -> Data? {
        let url = baseURL.appendingPathComponent(endpoint)

        var request = URLRequest(url: url)
        request.httpMethod = httpMethod

        if let contentType {
            request.setValue(contentType, forHTTPHeaderField: "Content-Type")
        }

        if let jsonData = jsonData {
            do {
                request.httpBody = try JSONSerialization.data(withJSONObject: jsonData)
            } catch {
                XCTFail("Failed to serialize JSON data")
            }
        }

        var requestResponse: URLResponse?
        var requestError: Error?
        var responseData: Data?

        let completionHandlerInvokedExpectation = XCTestExpectation(
            description: "Completion handler for the request is invoked"
        )

        let dataTask = URLSession.shared.dataTask(with: request) { data, response, error in
            requestResponse = response
            requestError = error

            guard let data = data,
                  let response = response as? HTTPURLResponse,
                  error == nil else {
                XCTFail("Error: \(error?.localizedDescription ?? "Unknown error")")
                return
            }

            if 200 ... 204 ~= response.statusCode && error == nil {
                responseData = data
            } else {
                XCTFail("Request failed")
            }

            completionHandlerInvokedExpectation.fulfill()
        }

        dataTask.resume()

        let waitResult = XCTWaiter.wait(for: [completionHandlerInvokedExpectation], timeout: 30)

        if waitResult != .completed {
            XCTFail("Failed to send packet capture API request - timeout")
        } else {
            if let response = requestResponse as? HTTPURLResponse {
                if (200 ... 201 ~= response.statusCode) == false {
                    XCTFail("Packet capture API request failed - unexpected server response")
                }
            }

            if let error = requestError {
                XCTFail("Packet capture API request failed - encountered error \(error.localizedDescription)")
            }
        }

        return responseData
    }
}
