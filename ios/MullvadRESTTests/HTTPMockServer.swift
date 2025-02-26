//
//  HTTPServer.swift
//  MullvadVPN
//
//  Created by Steffen Ernst on 2025-02-26.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//


import Foundation
import Network

@MainActor
class HTTPMockServer {
    let listener: NWListener
    let type: String
    let route: String
    let responseCode: UInt16
    let responseBody: String?

    init(type: String, route: String, responseCode: UInt16, responseBody: String? = nil) {
        let parameters = NWParameters.tcp
        // swiftlint:disable:next force_try
        self.listener = try! NWListener(using: parameters, on: 8080)
        self.type = type
        self.route = route
        self.responseBody = responseBody
        self.responseCode = responseCode

        self.listener.newConnectionHandler = { connection in
            Task {
                await self.handleConnection(connection)
            }
        }
        self.listener.start(queue: .main)
    }

    private func handleConnection(_ connection: NWConnection) {
        connection.start(queue: .main)

        connection.receive(minimumIncompleteLength: 1, maximumLength: 4096) { data, _, _, _ in
            if let data = data,
               let request = String(data: data, encoding: .utf8) {
                let requestComponents = request.split(separator: " ")
                let type = requestComponents[0]
                let route = requestComponents[1]

                Task {
                    guard type == self.type else {
                        await self.sendResponse(
                            to: connection,
                            responseCode: 501,
                            responseBody: nil
                        ) {
                            connection.cancel()
                        }
                        return
                    }
                    guard route == self.route else {
                        await self.sendResponse(
                            to: connection,
                            responseCode: 501,
                            responseBody: nil
                        ) {
                            connection.cancel()
                        }
                        return
                    }
                    await self.sendResponse(
                        to: connection,
                        responseCode: self.responseCode,
                        responseBody: self.responseBody
                    ) {
                        connection.cancel()
                    }
                }
            }
        }
    }

    private func sendResponse(
        to connection: NWConnection,
        responseCode: UInt16,
        responseBody: String?,
        completion: @Sendable @escaping () -> Void
    ) {
        let response = """
        HTTP/1.1 \(responseCode) OK\r\n
        \(responseBody ?? "")
        """

        if let data = response.data(using: .utf8) {
            connection.send(content: data, completion: .contentProcessed { error in
                if let error = error {
                    print("Error sending response: \(error)")
                }
                completion()
            })
        }
    }
}
