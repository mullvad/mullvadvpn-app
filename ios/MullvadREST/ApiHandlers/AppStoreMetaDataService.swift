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
    private let appStoreLink = URL(
        string: "https://itunes.apple.com/lookup?bundleId=net.mullvad.mullvadvpn"
    ).unsafelyUnwrapped

    private let localVersion: String =
        (Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String) ?? ""

    private var timer: DispatchSourceTimer? = DispatchSource.makeTimerSource()
    private let checkInterval: TimeInterval = Duration.days(1).timeInterval
    private let logger = Logger(label: "AppStoreMetaDataService")

    private let tunnelSettings: LatestTunnelSettings
    private let urlSession: URLSessionProtocol
    private let appPreferences: AppPreferences

    public init(tunnelSettings: LatestTunnelSettings, urlSession: URLSessionProtocol, appPreferences: AppPreferences) {
        self.tunnelSettings = tunnelSettings
        self.urlSession = urlSession
        self.appPreferences = appPreferences
    }

    public func scheduleTimer() {
        timer?.setEventHandler {
            Task { [weak self] in
                guard let self, tunnelSettings.includeAllNetworks else {
                    return
                }

                let newVersionExists = (try? await performVersionCheck()) ?? false
                if newVersionExists {
                    sendNotification()
                }
            }
        }

        // Resume deadline if there's time left from previous check. Otherwise, fire away.
        let deadline = max(checkInterval - appPreferences.lastVersionCheck.date.timeIntervalSinceNow, 0)

        timer?.schedule(deadline: .now() + deadline, repeating: .seconds(Int(checkInterval)))
        timer?.activate()
    }

    func performVersionCheck() async throws -> Bool {
        appPreferences.lastVersionCheck.date = .now

        let appStoreMetaData = try await fetchAppStoreMetaData()
        let appStoreVersion = appStoreMetaData?.version ?? ""

        if appStoreVersion.isNewerThan(localVersion) {
            appPreferences.lastVersionCheck.version = appStoreVersion
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

    private func sendNotification() {
        let content = UNMutableNotificationContent()
        content.title = NSLocalizedString("Update available", comment: "")
        content.body = String(
            format: NSLocalizedString(
                "Disable “%@” or disconnect before updating in order not to lose network connectivity.",
                comment: ""
            ),
            "Force all apps"
        )

        // When scheduling a user notification we need to make sure that the date has not passed
        // when it's actually added to the system. Giving it a few seconds leeway lets us be sure
        // that this is the case.
        let dateComponents = Calendar.current.dateComponents(
            [.second, .minute, .hour, .day, .month, .year],
            from: Date(timeIntervalSinceNow: 5)
        )
        let trigger = UNCalendarNotificationTrigger(dateMatching: dateComponents, repeats: false)

        let request = UNNotificationRequest(
            identifier: NotificationProviderIdentifier.newAppVersionSystemNotification.domainIdentifier,
            content: content,
            trigger: trigger
        )

        let identifier = request.identifier
        UNUserNotificationCenter.current().add(request) { [weak self, identifier] error in
            if let error {
                self?.logger.error(
                    "Failed to add notification request with identifier \(identifier). Error: \(error.description)"
                )
            }
        }
    }
}

extension String {
    /// Compares app versions, eg. "2025.10" > "2025.9" (= true).
    func isNewerThan(_ version: String) -> Bool {
        compare(version, options: .numeric) == .orderedDescending
    }
}
