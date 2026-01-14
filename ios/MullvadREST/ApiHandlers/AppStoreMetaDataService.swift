//
//  AppStoreMetaDataService.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-01-09.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadSettings
import MullvadTypes
import UserNotifications

public final class AppStoreMetaDataService: @unchecked Sendable {
    private let appStoreLink = URL(string: "https://itunes.apple.com/lookup?bundleId=net.mullvad.mullvadvpn")
    private let checkInterval: TimeInterval = 86_400  // 24 hours
    private let logger = Logger(label: "AppStoreMetaDataService")

    public let urlSession: URLSessionProtocol
    public let appPreferences: AppPreferences

    private var localVersion: String {
        (Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String) ?? ""
    }

    public init(urlSession: URLSessionProtocol, appPreferences: AppPreferences) {
        self.urlSession = urlSession
        self.appPreferences = appPreferences
    }

    public func performVersionCheck() async throws -> Bool {
        appPreferences.lastVersionCheckDate = .now

        let appStoreMetaData = try await fetchAppStoreMetaData()
        let appStoreVersion = appStoreMetaData?.version ?? ""

        if appStoreVersion.isNewerThan(localVersion) {
            appPreferences.lastVersionCheckVersion = appStoreVersion
            return true
        }

        return false
    }

    private func fetchAppStoreMetaData() async throws -> AppStoreMetaData? {
        do {
            let data = try await urlSession.data(
                for: URLRequest(url: appStoreLink, timeoutInterval: REST.defaultAPINetworkTimeout.timeInterval)
            )
            return try JSONDecoder().decode(AppStoreMetaData.self, from: data.0)
        } catch {
            logger.log(level: .error, "Could not fetch App Store metadata: \(error.description)")
        }

        return nil
    }
}

extension String {
    /// Compares app versions, eg. "2025.10" > "2025.9" (= true).
    func isNewerThan(_ version: String) -> Bool {
        compare(version, options: .numeric) == .orderedDescending
    }
}
