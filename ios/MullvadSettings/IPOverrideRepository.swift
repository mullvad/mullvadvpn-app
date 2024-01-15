//
//  IPOverrideRepository.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-16.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging

public protocol IPOverrideRepositoryProtocol {
    func add(_ overrides: [IPOverride])
    func fetchAll() -> [IPOverride]
    func deleteAll()
    func parse(data: Data) throws -> [IPOverride]
}

public class IPOverrideRepository: IPOverrideRepositoryProtocol {
    private let logger = Logger(label: "IPOverrideRepository")
    private let readWriteLock = NSLock()

    public init() {}

    public func add(_ overrides: [IPOverride]) {
        var storedOverrides = fetchAll()

        overrides.forEach { override in
            if let existingOverrideIndex = storedOverrides.firstIndex(where: { $0.hostname == override.hostname }) {
                var existingOverride = storedOverrides[existingOverrideIndex]

                if let ipv4Address = override.ipv4Address {
                    existingOverride.ipv4Address = ipv4Address
                }

                if let ipv6Address = override.ipv6Address {
                    existingOverride.ipv6Address = ipv6Address
                }

                storedOverrides[existingOverrideIndex] = existingOverride
            } else {
                storedOverrides.append(override)
            }
        }

        do {
            try writeIpOverrides(storedOverrides)
        } catch {
            logger.error("Could not add override(s): \(overrides) \nError: \(error)")
        }
    }

    public func fetchAll() -> [IPOverride] {
        return (try? readIpOverrides()) ?? []
    }

    public func deleteAll() {
        do {
            try readWriteLock.withLock {
                try SettingsManager.store.delete(key: .ipOverrides)
            }
        } catch {
            logger.error("Could not delete all overrides. \nError: \(error)")
        }
    }

    public func parse(data: Data) throws -> [IPOverride] {
        let decoder = JSONDecoder()
        let jsonData = try decoder.decode(RelayOverrides.self, from: data)

        return jsonData.overrides
    }

    private func readIpOverrides() throws -> [IPOverride] {
        try readWriteLock.withLock {
            let parser = makeParser()
            let data = try SettingsManager.store.read(key: .ipOverrides)
            return try parser.parseUnversionedPayload(as: [IPOverride].self, from: data)
        }
    }

    private func writeIpOverrides(_ overrides: [IPOverride]) throws {
        let parser = makeParser()
        let data = try parser.produceUnversionedPayload(overrides)

        try readWriteLock.withLock {
            try SettingsManager.store.write(data, for: .ipOverrides)
        }
    }

    private func makeParser() -> SettingsParser {
        SettingsParser(decoder: JSONDecoder(), encoder: JSONEncoder())
    }
}
