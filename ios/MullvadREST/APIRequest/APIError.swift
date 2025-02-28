//
//  APIError.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-02-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

public struct APIError: Error, Codable, Sendable {
    public let statusCode: Int
    public let errorDescription: String
    public let serverResponseCode: String?

    public init(statusCode: Int, errorDescription: String, serverResponseCode: String?) {
        self.statusCode = statusCode
        self.errorDescription = errorDescription
        self.serverResponseCode = serverResponseCode
    }
}
