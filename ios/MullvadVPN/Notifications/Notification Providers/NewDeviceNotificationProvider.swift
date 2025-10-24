//
//  NewDeviceNotificationProvider.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-04-21.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import UIKit.UIColor
import UIKit.UIFont

final class NewDeviceNotificationProvider: NotificationProvider,
    InAppNotificationProvider, @unchecked Sendable
{
    // MARK: - private properties

    private let tunnelManager: TunnelManager

    private var storedDeviceData: StoredDeviceData? {
        tunnelManager.deviceState.deviceData
    }

    private var tunnelObserver: TunnelBlockObserver?
    private var isNewDeviceRegistered = false

    private var attributedBody: NSAttributedString {
        let formattedString = NSLocalizedString(
            "This device is now named **%@**. See more under \"Manage devices\" in Account.",
            comment: ""
        )
        let deviceName = storedDeviceData?.capitalizedName ?? ""
        let string = String(format: formattedString, deviceName)

        return NSAttributedString(
            markdownString: string,
            options: MarkdownStylingOptions(
                font: .preferredFont(forTextStyle: .subheadline)
            )
        ) { _, _ in
            [.foregroundColor: UIColor.InAppNotificationBanner.titleColor]
        }
    }

    // MARK: - public properties

    var notificationDescriptor: InAppNotificationDescriptor? {
        guard isNewDeviceRegistered else { return nil }
        return InAppNotificationDescriptor(
            identifier: identifier,
            style: .success,
            title: NSLocalizedString("NEW DEVICE CREATED", comment: ""),
            body: attributedBody,
            button: InAppNotificationAction(
                image: UIImage.Buttons.closeSmall,
                handler: { [weak self] in
                    guard let self else { return }
                    isNewDeviceRegistered = false
                    sendAction()
                    invalidate()
                }
            )
        )
    }

    // MARK: - initialize

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
        super.init()
        addObservers()
    }

    override var identifier: NotificationProviderIdentifier {
        .registeredDeviceInAppNotification
    }

    override var priority: NotificationPriority {
        .medium
    }

    private func addObservers() {
        tunnelObserver =
            TunnelBlockObserver(didUpdateDeviceState: { [weak self] _, deviceState, previousDeviceState in
                if previousDeviceState == .loggedOut,
                    case .loggedIn = deviceState
                {
                    self?.isNewDeviceRegistered = true
                    DispatchQueue.main.asyncAfter(deadline: .now() + .seconds(1)) { [weak self] in
                        self?.invalidate()
                    }

                } else if case .loggedIn = previousDeviceState,
                    deviceState == .loggedOut || deviceState == .revoked
                {
                    self?.isNewDeviceRegistered = false
                    self?.invalidate()
                }
            })
        tunnelObserver.flatMap { tunnelManager.addObserver($0) }
    }
}
