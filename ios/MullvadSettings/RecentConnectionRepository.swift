//
//  RecentConnectionRepository.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-09-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadLogging

public struct RecentConnectionRepository: RecentConnectionRepositoryProtocol {
    private let logger = Logger(label: "RecentConnectionRepository")
    private let maxLimit: Int
    private let settingsParser: SettingsParser = {
        SettingsParser(decoder: JSONDecoder(), encoder: JSONEncoder())
    }()

    init(maxLimit: Int = 50) {
        self.maxLimit = maxLimit
    }

    public func add(_ recentConnection: RecentConnection) throws {
        do {
            var recentConnections = try all()
            if let index = recentConnections.firstIndex(of: recentConnection) {
                recentConnections.remove(at: index)
            }
            recentConnections.insert(recentConnection, at: 0)

            // keep at most max items (safe even if the list has fewer than max)
            try write(Array(recentConnections.prefix(maxLimit)))
        } catch {
            logger.error(error: error)
        }
    }

    public func clear() throws {
        do {
            try write([])
        } catch {
            logger.error(error: error)
        }
    }

    public func all() throws -> [RecentConnection] {
        try read()
    }
}

private extension RecentConnectionRepository {
    private func read() throws -> [RecentConnection] {
        let data = try SettingsManager.store.read(key: .recentConnections)

        return try settingsParser.parseUnversionedPayload(as: [RecentConnection].self, from: data)
    }

    private func write(_ list: [RecentConnection]) throws {
        let data = try settingsParser.produceUnversionedPayload(list)
        try SettingsManager.store.write(data, for: .recentConnections)
    }
}
