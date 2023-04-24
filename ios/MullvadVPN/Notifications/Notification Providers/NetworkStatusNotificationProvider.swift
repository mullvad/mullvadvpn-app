//
//  NetworkStatusNotificationProvider.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-04-24.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

final class NetworkStatusNotificationProvider: NotificationProvider, InAppNotificationProvider {
    private var lastStatus: NWPath.Status?
    private let networkMonitor = NWPathMonitor()

    static let identifier = "net.mullvad.MullvadVPN.NetworkStatusNotificationProvider"

    override var identifier: String {
        return Self.identifier
    }

    var notificationDescriptor: InAppNotificationDescriptor? {
        if lastStatus == .unsatisfied {
            return networkNotificationDescription()
        } else {
            return nil
        }
    }

    override init() {
        super.init()

        networkMonitor.pathUpdateHandler = { [weak self] path in
            guard self?.lastStatus != path.status else { return }

            self?.lastStatus = path.status
            self?.invalidate()
        }

        networkMonitor.start(queue: .global(qos: .background))
    }

    private func networkNotificationDescription() -> InAppNotificationDescriptor {
        return InAppNotificationDescriptor(
            identifier: identifier,
            style: .warning,
            title: NSLocalizedString(
                "TUNNEL_NO_NETWORK_INAPP_NOTIFICATION_TITLE",
                value: "NETWORK ISSUES",
                comment: ""
            ),
            body: NSLocalizedString(
                "TUNNEL_NO_NETWORK_INAPP_NOTIFICATION_BODY",
                value: """
                Your device is offline. Try connecting again when the device \
                has internet access.
                """,
                comment: ""
            )
        )
    }
}
