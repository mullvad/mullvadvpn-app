//
//  LaunchArguments.swift
//  MullvadTypes
//
//  Created by Mojgan on 2024-05-06.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

public protocol Taggable {
    static var tag: String { get }
}

public extension Taggable {
    static var tag: String {
        String(describing: self)
    }
}

public enum MullvadExecutableTarget: Codable, Sendable {
    case uiTests, screenshots, main

    var isUITest: Bool {
        self == .uiTests || self == .screenshots
    }
}
// This arguments are picked up in AppDelegate.
public struct LaunchArguments: Codable, Taggable, Sendable {
    public enum AuthenticationState: String, Codable, Sendable {
        case keepLoggedIn
        case forceLoggedOut
    }

    public enum LocalDataResetPolicy: String, Codable, Sendable {
        case none
        case all
    }

    // Defines which target is running
    public var target: MullvadExecutableTarget = .main

    // Disable animations to speed up tests.
    public var areAnimationsDisabled = false

    // Controls login state at launch.
    public var authenticationState: AuthenticationState = .keepLoggedIn

    // Controls which local data is cleared at launch.
    public var localDataResetPolicy: LocalDataResetPolicy = .none
}

public extension ProcessInfo {
    func decode<T: Taggable & Decodable>(_: T.Type) throws -> T {
        guard let environment = environment[T.tag] else {
            throw DecodingError.valueNotFound(
                T.self,
                DecodingError.Context(codingPath: [], debugDescription: "\(T.self) not found in environment")
            )
        }
        return try T.decode(from: environment)
    }
}

extension Encodable {
    public func toJSON(_ encoder: JSONEncoder = JSONEncoder()) throws -> String {
        let data = try encoder.encode(self)
        guard let result = String(bytes: data, encoding: .utf8) else {
            throw EncodingError.invalidValue(
                self,
                EncodingError.Context(codingPath: [], debugDescription: "Could not encode self to a utf-8 string")
            )
        }
        return result
    }
}

private extension Decodable {
    static func decode(from json: String) throws -> Self {
        guard let data = json.data(using: .utf8) else {
            throw DecodingError.valueNotFound(
                Self.self,
                DecodingError.Context(
                    codingPath: [],
                    debugDescription: "Could not convert \(json) into UTF-8 Data"
                )
            )
        }

        return try JSONDecoder().decode(self, from: data)
    }
}
