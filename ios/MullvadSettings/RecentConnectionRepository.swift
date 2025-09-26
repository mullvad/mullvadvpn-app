//
//  RecentConnectionRepository.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-09-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadLogging

public actor RecentConnectionRepository: RecentConnectionRepositoryProtocol {
    private let logger = Logger(label: "RecentConnectionRepository")

    private let settingsParser: SettingsParser = {
        SettingsParser(decoder: JSONDecoder(), encoder: JSONEncoder())
    }()

    public func add(_ recentConnection: RecentConnection, maxLimit: Int = 50) {
        do {
            var recentConnections = all()
            if let index = recentConnections.firstIndex(of: recentConnection) {
                recentConnections.remove(at: index)
            }
            recentConnections.append(recentConnection)

            // Sort connections descending by `lastSelected` (most recent first)
            // and keep at most max items (safe even if the list has fewer than max)
            try write(Array(recentConnections.sorted(by: { $0.lastSelected > $1.lastSelected }).prefix(maxLimit)))
        } catch {
            logger.error(error: error)
        }
    }

    public func clear() {
        do {
            try write([])
        } catch {
            logger.error(error: error)
        }
    }

    public func all() -> [RecentConnection] {
        (try? read()) ?? []
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
