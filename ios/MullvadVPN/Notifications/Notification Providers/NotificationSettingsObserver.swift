//
//  NotificationSettingsObserver.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-02-06.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import MullvadSettings
import MullvadTypes

//MARK: - NotificationSettingsPropagation
protocol NotificationSettingsPropagation: Sendable {
    typealias NotificationSettingsHandler = (NotificationSettings) -> Void
    var onNewSettings: NotificationSettingsHandler? { get set }
}

protocol NotificationSettingsObserver: AnyObject {
    func didUpdateNotificationSettings(_ settings: NotificationSettings)
}

class NotificationSettingsObserverBlock: NotificationSettingsObserver {
    typealias DidUpdateNotificationSettingsHandler = (NotificationSettings) -> Void
    var onNewSettings: DidUpdateNotificationSettingsHandler

    init(didUpdateSettings: @escaping DidUpdateNotificationSettingsHandler) {
        self.onNewSettings = didUpdateSettings
    }

    func didUpdateNotificationSettings(_ settings: NotificationSettings) {
        self.onNewSettings(settings)
    }
}

final class NotificationSettingsListener: NotificationSettingsPropagation, @unchecked Sendable {
    var onNewSettings: NotificationSettingsHandler?

    init(onNewSettings: NotificationSettingsHandler? = nil) {
        self.onNewSettings = onNewSettings
    }
}

final class NotificationSettingsUpdater: Sendable {
    /// Observers.
    private let observerList = ObserverList<NotificationSettingsObserver>()
    nonisolated(unsafe) private var listener: NotificationSettingsPropagation

    init(listener: NotificationSettingsPropagation) {
        self.listener = listener
        self.listener.onNewSettings = { [weak self] settings in
            guard let self else { return }
            self.observerList.notify {
                $0.didUpdateNotificationSettings(settings)
            }
        }
    }

    // MARK: - Multihop observations

    func addObserver(_ observer: NotificationSettingsObserver) {
        observerList.append(observer)
    }

    func removeObserver(_ observer: NotificationSettingsObserver) {
        observerList.remove(observer)
    }
}
