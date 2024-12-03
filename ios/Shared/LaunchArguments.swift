//
//  LaunchArguments.swift
//  MullvadTypes
//
//  Created by Mojgan on 2024-05-06.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
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

public enum MullvadExecutableTarget: Codable {
    case uiTests, screenshots, main
}

// This arguments are picked up in AppDelegate.
public struct LaunchArguments: Codable, Taggable {
    // Defines which target is running
    public var target: MullvadExecutableTarget = .main

    // Disable animations to speed up tests.
    public var areAnimationsDisabled = false
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
