//
//  NotificationManager.swift
//  MullvadVPN
//
//  Created by pronebird on 31/05/2021.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadTypes
@preconcurrency import UserNotifications

final class NotificationManager: NotificationProviderDelegate {
    private let logger = Logger(label: "NotificationManager")
    private var _notificationProviders: [NotificationProvider] = []
    private var inAppNotificationDescriptors: [InAppNotificationDescriptor] = []

    var notificationProviders: [NotificationProvider] {
        get {
            _notificationProviders
        }
        set(newNotificationProviders) {
            dispatchPrecondition(condition: .onQueue(.main))

            for oldNotificationProvider in _notificationProviders {
                oldNotificationProvider.delegate = nil
            }

            for newNotificationProvider in newNotificationProviders {
                newNotificationProvider.delegate = self
            }

            _notificationProviders = newNotificationProviders.sorted { $0.priority > $1.priority }
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

    nonisolated(unsafe) static let shared = NotificationManager()

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
                    pendingRequestIdentifiersToRemove.append(notificationProvider.identifier.domainIdentifier)
                }

                if notificationProvider.shouldRemoveDeliveredRequests {
                    deliveredRequestIdentifiersToRemove.append(notificationProvider.identifier.domainIdentifier)
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

        let logger = self.logger
        Task {
            let isAllowed = await UNUserNotificationCenter.isAllowed
            guard isAllowed else { return }
            for newRequest in newSystemNotificationRequests {
                do {
                    try await notificationCenter.add(newRequest)
                } catch {
                    logger.error(
                        error: error,
                        message: "Failed to add notification request with identifier \(newRequest.identifier)."
                    )
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
        dispatchPrecondition(condition: .onQueue(.main))

        let requestIdentifier = response.notification.request.identifier.split(separator: ".").last
        guard let sourceProvider = NotificationProviderIdentifier(rawValue: String(requestIdentifier ?? "")) else {
            logger.warning(
                "Received response with request identifier: \(requestIdentifier ?? "-") that didn't map to any notification provider"
            )
            return
        }

        let notificationResponse = NotificationResponse(
            providerIdentifier: sourceProvider,
            actionIdentifier: response.actionIdentifier,
            systemResponse: response
        )

        delegate?.notificationManager(self, didReceiveResponse: notificationResponse)
    }

    // MARK: - NotificationProviderDelegate

    func notificationProviderDidInvalidate(_ notificationProvider: NotificationProvider) {
        dispatchPrecondition(condition: .onQueue(.main))

        // Invalidate system notification
        if let notificationProvider = notificationProvider as? SystemNotificationProvider {
            let notificationCenter = UNUserNotificationCenter.current()

            if notificationProvider.shouldRemovePendingRequests {
                notificationCenter.removePendingNotificationRequests(withIdentifiers: [
                    notificationProvider.identifier.domainIdentifier
                ])
            }

            if notificationProvider.shouldRemoveDeliveredRequests {
                notificationCenter.removeDeliveredNotifications(withIdentifiers: [
                    notificationProvider.identifier.domainIdentifier
                ])
            }

            let logger = self.logger
            if let request = notificationProvider.notificationRequest {
                Task { @MainActor in
                    guard await UNUserNotificationCenter.isAllowed else { return }
                    do {
                        try await notificationCenter.add(request)
                    } catch {
                        logger.error(
                            """
                            Failed to add notification request with identifier \
                            \(request.identifier). Error: \(error.description)
                            """)
                    }
                }
            }
        }

        invalidateInAppNotification(notificationProvider)
    }

    private func invalidateInAppNotification(_ notificationProvider: NotificationProvider) {
        if let notificationProvider = notificationProvider as? InAppNotificationProvider {
            var newNotificationDescriptors = inAppNotificationDescriptors

            if let replaceNotificationDescriptor = notificationProvider.notificationDescriptor {
                newNotificationDescriptors =
                    notificationProviders
                    .compactMap { notificationProvider -> InAppNotificationDescriptor? in
                        if replaceNotificationDescriptor.identifier == notificationProvider.identifier {
                            return replaceNotificationDescriptor
                        } else {
                            return inAppNotificationDescriptors.first { descriptor in
                                descriptor.identifier == notificationProvider.identifier
                            }
                        }
                    }
            } else {
                newNotificationDescriptors.removeAll { descriptor in
                    descriptor.identifier == notificationProvider.identifier
                }
            }

            inAppNotificationDescriptors = newNotificationDescriptors

            delegate?.notificationManagerDidUpdateInAppNotifications(
                self,
                notifications: inAppNotificationDescriptors
            )
        }
    }

    func notificationProvider(_ notificationProvider: NotificationProvider, didReceiveAction actionIdentifier: String) {
        dispatchPrecondition(condition: .onQueue(.main))

        let notificationResponse = NotificationResponse(
            providerIdentifier: notificationProvider.identifier,
            actionIdentifier: actionIdentifier
        )

        delegate?.notificationManager(self, didReceiveResponse: notificationResponse)
    }
}
