//
//  RegisteredDeviceInAppNotification.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-04-21.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit.UIColor
import UIKit.UIFont

final class RegisteredDeviceInAppNotification: NotificationProvider, InAppNotificationProvider {
    // MARK: - private properties

    private let tunnelManager: TunnelManager

    private var shouldShowBanner = false
    private var deviceState: DeviceState
    private var tunnelObserver: TunnelBlockObserver?

    private var attributedBody: NSAttributedString {
        guard case let .loggedIn(_, storedDeviceData) = deviceState else { return .init(string: "") }
        let formattedString = NSLocalizedString(
            "ACCOUNT_CREATION_INAPP_NOTIFICATION_BODY",
            value: "Welcome, this device is now called **%@**. For more details see the info button in Account.",
            comment: ""
        )
        let deviceName = storedDeviceData.capitalizedName
        let string = String(format: formattedString, deviceName)
        return NSMutableAttributedString(markdownString: string, font: .systemFont(ofSize: 14.0)) { deviceName in
            return [.foregroundColor: UIColor.InAppNotificationBanner.titleColor]
        }
    }

    // MARK: - public properties

    var notificationDescriptor: InAppNotificationDescriptor? {
        guard shouldShowBanner else { return nil }
        return InAppNotificationDescriptor(
            identifier: identifier,
            style: .success,
            title: NSLocalizedString(
                "ACCOUNT_CREATION_INAPP_NOTIFICATION_TITLE",
                value: "NEW DEVICE CREATED",
                comment: ""
            ),
            body: attributedBody,
            action: .init(
                image: .init(named: "IconCloseSml"),
                handler: { [weak self] in
                    guard let self else { return }

                    sendAction()

                    shouldShowBanner = false
                    invalidate()
                }
            )
        )
    }

    // MARK: - initialize

    static let identifier = "net.mullvad.MullvadVPN.RegisteredDeviceInAppNotification"

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
        deviceState = tunnelManager.deviceState
        super.init()
        addObservers()
    }

    override var identifier: String {
        return Self.identifier
    }

    private func addObservers() {
        tunnelObserver =
            TunnelBlockObserver(didUpdateDeviceState: { [weak self] tunnelManager, deviceState, previousDeviceState in
                guard let self, case .loggedIn = deviceState else { return }

                shouldShowBanner = true
                self.deviceState = deviceState
                invalidate()
            })
        tunnelObserver.flatMap { tunnelManager.addObserver($0) }
    }
}
