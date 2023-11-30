//
//  RegisteredDeviceInAppNotification.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-04-21.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import UIKit.UIColor
import UIKit.UIFont

final class RegisteredDeviceInAppNotificationProvider: NotificationProvider,
    InAppNotificationProvider {
    // MARK: - private properties

    private let tunnelManager: TunnelManager

    private var storedDeviceData: StoredDeviceData? {
        tunnelManager.deviceState.deviceData
    }

    private var tunnelObserver: TunnelBlockObserver?
    private var isNewDeviceRegistered = false

    private var attributedBody: NSAttributedString {
        let formattedString = NSLocalizedString(
            "ACCOUNT_CREATION_INAPP_NOTIFICATION_BODY",
            value: "Welcome, this device is now called **%@**. For more details see the info button in Account.",
            comment: ""
        )
        let deviceName = storedDeviceData?.capitalizedName ?? ""
        let string = String(format: formattedString, deviceName)

        let stylingOptions = MarkdownStylingOptions(font: .systemFont(ofSize: 14.0))

        return NSAttributedString(markdownString: string, options: stylingOptions, applyEffect: { markdownType, _ in
            if case .bold = markdownType {
                return [.foregroundColor: UIColor.InAppNotificationBanner.titleColor]
            } else {
                return [:]
            }
        })
    }

    // MARK: - public properties

    var notificationDescriptor: InAppNotificationDescriptor? {
        guard isNewDeviceRegistered else { return nil }
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

    private func addObservers() {
        tunnelObserver =
            TunnelBlockObserver(didUpdateDeviceState: { [weak self] _, deviceState, previousDeviceState in
                if previousDeviceState == .loggedOut,
                   case .loggedIn = deviceState {
                    self?.isNewDeviceRegistered = true
                    DispatchQueue.main.asyncAfter(deadline: .now() + .seconds(1)) { [weak self] in
                        self?.invalidate()
                    }

                } else if case .loggedIn = previousDeviceState,
                          deviceState == .loggedOut || deviceState == .revoked {
                    self?.isNewDeviceRegistered = false
                    self?.invalidate()
                }
            })
        tunnelObserver.flatMap { tunnelManager.addObserver($0) }
    }
}
