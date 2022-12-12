//
//  NotificationManager.swift
//  MullvadVPN
//
//  Created by pronebird on 31/05/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import UserNotifications

final class NotificationManager: NotificationProviderDelegate {
    private lazy var logger = Logger(label: "NotificationManager")
    private var _notificationProviders: [NotificationProvider] = []
    private var inAppNotificationDescriptors: [InAppNotificationDescriptor] = []

    var notificationProviders: [NotificationProvider] {
        set(newNotificationProviders) {
            dispatchPrecondition(condition: .onQueue(.main))

            for oldNotificationProvider in _notificationProviders {
                oldNotificationProvider.delegate = nil
            }

            for newNotificationProvider in newNotificationProviders {
                newNotificationProvider.delegate = self
            }

            _notificationProviders = newNotificationProviders
        }
        get {
            return _notificationProviders
        }
    }

    weak var delegate: NotificationManagerDelegate? {
        didSet {
            dispatchPrecondition(condition: .onQueue(.main))

            // Pump in-app notifications when changing delegate.
            if !inAppNotificationDescriptors.isEmpty {
                delegate?.notificationManagerDidUpdateInAppNotifications(
                    self,
                    notifications: inAppNotificationDescriptors
                )
            }
        }
    }

    static let shared = NotificationManager()

    private init() {}

    func updateNotifications() {
        dispatchPrecondition(condition: .onQueue(.main))

        var newSystemNotificationRequests = [UNNotificationRequest]()
        var newInAppNotificationDescriptors = [InAppNotificationDescriptor]()
        var pendingRequestIdentifiersToRemove = [String]()
        var deliveredRequestIdentifiersToRemove = [String]()

        for notificationProvider in notificationProviders {
            if let notificationProvider = notificationProvider as? SystemNotificationProvider {
                if notificationProvider.shouldRemovePendingRequests {
                    pendingRequestIdentifiersToRemove.append(notificationProvider.identifier)
                }

                if notificationProvider.shouldRemoveDeliveredRequests {
                    deliveredRequestIdentifiersToRemove.append(notificationProvider.identifier)
                }

                if let request = notificationProvider.notificationRequest {
                    newSystemNotificationRequests.append(request)
                }
            }

            if let notificationProvider = notificationProvider as? InAppNotificationProvider {
                if let descriptor = notificationProvider.notificationDescriptor {
                    newInAppNotificationDescriptors.append(descriptor)
                }
            }
        }

        let notificationCenter = UNUserNotificationCenter.current()

        notificationCenter.removePendingNotificationRequests(
            withIdentifiers: pendingRequestIdentifiersToRemove
        )
        notificationCenter.removeDeliveredNotifications(
            withIdentifiers: deliveredRequestIdentifiersToRemove
        )

        requestNotificationPermissions { granted in
            guard granted else { return }

            for newRequest in newSystemNotificationRequests {
                notificationCenter.add(newRequest) { error in
                    if let error = error {
                        self.logger.error(
                            error: error,
                            message: """
                            Failed to add notification request with identifier \
                            \(newRequest.identifier).
                            """
                        )
                    }
                }
            }
        }

        inAppNotificationDescriptors = newInAppNotificationDescriptors

        delegate?.notificationManagerDidUpdateInAppNotifications(
            self,
            notifications: newInAppNotificationDescriptors
        )
    }

    func handleSystemNotificationResponse(_ response: UNNotificationResponse) {
        for case let notificationProvider as SystemNotificationProvider in notificationProviders {
            if notificationProvider.handleResponse(response) {
                return
            }
        }
    }

    // MARK: - Private

    private func requestNotificationPermissions(completion: @escaping (Bool) -> Void) {
        let authorizationOptions: UNAuthorizationOptions = [.alert, .sound, .provisional]
        let userNotificationCenter = UNUserNotificationCenter.current()

        userNotificationCenter.getNotificationSettings { notificationSettings in
            switch notificationSettings.authorizationStatus {
            case .notDetermined:
                userNotificationCenter.requestAuthorization(
                    options: authorizationOptions
                ) { granted, error in
                    if let error = error {
                        self.logger.error(
                            error: error,
                            message: "Failed to obtain user notifications authorization"
                        )
                    }
                    completion(granted)
                }

            case .authorized, .provisional:
                completion(true)

            case .denied, .ephemeral:
                fallthrough

            @unknown default:
                completion(false)
            }
        }
    }

    // MARK: - NotificationProviderDelegate

    func notificationProviderDidInvalidate(_ notificationProvider: NotificationProvider) {
        dispatchPrecondition(condition: .onQueue(.main))

        // Invalidate system notification
        if let notificationProvider = notificationProvider as? SystemNotificationProvider {
            let notificationCenter = UNUserNotificationCenter.current()

            if notificationProvider.shouldRemovePendingRequests {
                notificationCenter.removePendingNotificationRequests(withIdentifiers: [
                    notificationProvider.identifier,
                ])
            }

            if notificationProvider.shouldRemoveDeliveredRequests {
                notificationCenter.removeDeliveredNotifications(withIdentifiers: [
                    notificationProvider.identifier,
                ])
            }

            if let request = notificationProvider.notificationRequest {
                requestNotificationPermissions { granted in
                    guard granted else { return }

                    notificationCenter.add(request) { error in
                        if let error = error {
                            self.logger.error(
                                """
                                Failed to add notification request with identifier \
                                \(request.identifier). Error: \(error.localizedDescription)
                                """
                            )
                        }
                    }
                }
            }
        }

        // Invalidate in-app notification
        if let notificationProvider = notificationProvider as? InAppNotificationProvider {
            var newNotificationDescriptors = inAppNotificationDescriptors

            if let replaceNotificationDescriptor = notificationProvider.notificationDescriptor {
                newNotificationDescriptors = notificationProviders
                    .compactMap { notificationProvider -> InAppNotificationDescriptor? in
                        if replaceNotificationDescriptor.identifier == notificationProvider
                            .identifier
                        {
                            return replaceNotificationDescriptor
                        } else {
                            return inAppNotificationDescriptors.first { descriptor in
                                return descriptor.identifier == notificationProvider.identifier
                            }
                        }
                    }
            } else {
                newNotificationDescriptors.removeAll { descriptor in
                    return descriptor.identifier == notificationProvider.identifier
                }
            }

            inAppNotificationDescriptors = newNotificationDescriptors

            delegate?.notificationManagerDidUpdateInAppNotifications(
                self,
                notifications: inAppNotificationDescriptors
            )
        }
    }
}
