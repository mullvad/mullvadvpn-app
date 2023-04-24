//
//  AccountCreationInAppNotification.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-04-21.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit.UIColor
import UIKit.UIFont
final class RegisteredDeviceInAppNotification: NotificationProvider,
    InAppNotificationProvider
{
    typealias CompletionHandler = (DeviceState) -> Void

    private let tunnelManager: TunnelManager
    private let completionHandler: CompletionHandler?

    private var shouldShowBanner = false
    private var deviceState: DeviceState {
        didSet {}
    }

    private var tunnelObserver: TunnelBlockObserver?

    // MARK: - computed properties

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
                    guard let self = self else { return }
                    self.shouldShowBanner = false
                    self.invalidate()
                    self.completionHandler?(self.deviceState)
                }
            )
        )
    }

    // MARK: - initialize

    init(tunnelManager: TunnelManager, completionHandler: CompletionHandler? = nil) {
        self.tunnelManager = tunnelManager
        self.completionHandler = completionHandler
        deviceState = tunnelManager.deviceState
        super.init()
        addObservers()
    }

    override var identifier: String {
        "net.mullvad.MullvadVPN.AccountCreationInAppNotification"
    }

    private func addObservers() {
        tunnelObserver = TunnelBlockObserver(didUpdateDeviceState: { [weak self] tunnelManager, deviceState in
            guard let self = self,
                  case .loggedIn = deviceState else { return }
            self.shouldShowBanner = true
            self.deviceState = deviceState
            self.invalidate()
        })
        tunnelManager.addObserver(tunnelObserver!)
    }
}
