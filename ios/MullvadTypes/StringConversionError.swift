//
//  StringDecodingError.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-02.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

public struct StringDecodingError: LocalizedError {
    public let data: Data

    public init(data: Data) {
        self.data = data
    }

    public var errorDescription: String? {
        "Failed to decode string from data."
    }
}

public struct StringEncodingError: LocalizedError {
    public let string: String

    public init(string: String) {
        self.string = string
    }

    public var errorDescription: String? {
        "Failed to encode string into data."
    }
}
