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
    public var isDisabledAnimations = false
}

public extension ProcessInfo {
    func decode<T: Taggable & Decodable>(_: T.Type) -> T? {
        guard
            let environment = environment[T.tag],
            let codable = T.decode(from: environment) else {
            return nil
        }

        return codable
    }
}

extension Encodable {
    public func toJSON(_ encoder: JSONEncoder = JSONEncoder()) throws -> String {
        let data = try encoder.encode(self)
        let result = String(decoding: data, as: UTF8.self)
        return result
    }
}

private extension Decodable {
    static func decode(from json: String) -> Self? {
        guard let data = json.data(using: .utf8) else {
            return nil
        }

        return try? JSONDecoder().decode(self, from: data)
    }
}
