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

/// Represents a packet in packet capture
struct Packet: Codable {
    let fromPeer: Bool
    let timestamp: Int64

    enum CodingKeys: String, CodingKey {
        case fromPeer = "from_peer"
        case timestamp
    }
}

/// Represents a stream in packet capture
struct Stream: Codable {
    let peerAddr: String
    let otherAddr: String
    let flowID: String?
    let transportProtocol: NetworkTransportProtocol
    let packets: [Packet]

    enum CodingKeys: String, CodingKey {
        case peerAddr = "peer_addr"
        case otherAddr = "other_addr"
        case flowID = "flow_id"
        case transportProtocol = "transport_protocol"
        case packets
    }
}

class PacketCaptureAPIClient {
    // swiftlint:disable force_cast
    let baseURL = URL(
        string: Bundle(for: PacketCaptureAPIClient.self)
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
