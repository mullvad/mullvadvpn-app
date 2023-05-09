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

final class RegisteredDeviceInAppNotificationProvider: NotificationProvider,
    InAppNotificationProvider
{
    static let identifier = "net.mullvad.MullvadVPN.RegisteredDeviceInAppNotification"

    // MARK: - private properties

    private let tunnelManager: TunnelManager
    private var showsInAppNotification = false {
        didSet {
            DispatchQueue.main.asyncAfter(deadline: .now() + .seconds(1), execute: .init(block: { [weak self] in
                self?.invalidate()
            }))
        }
    }

    private var storedDeviceData: StoredDeviceData?
    private var tunnelObserver: TunnelBlockObserver?

    private var attributedBody: NSAttributedString {
        let formattedString = NSLocalizedString(
            "ACCOUNT_CREATION_INAPP_NOTIFICATION_BODY",
            value: "Welcome, this device is now called **%@**. For more details see the info button in Account.",
            comment: ""
        )
        let deviceName = storedDeviceData?.capitalizedName ?? ""
        let string = String(format: formattedString, deviceName)
        return NSMutableAttributedString(markdownString: string, font: .systemFont(ofSize: 14.0)) { deviceName in
            return [.foregroundColor: UIColor.InAppNotificationBanner.titleColor]
        }
    }

    // MARK: - public properties

    var notificationDescriptor: InAppNotificationDescriptor? {
        guard showsInAppNotification else { return nil }
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
                    self.showsInAppNotification = false
                    self.invalidate()
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

    override var identifier: String {
        Self.identifier
    }

    private func addObservers() {
        tunnelObserver =
            TunnelBlockObserver(didUpdateDeviceState: { [weak self] tunnelManager, deviceState, previousDeviceState in
                if previousDeviceState == .loggedOut,
                   case .loggedIn = deviceState
                {
                    self?.storedDeviceData = deviceState.deviceData
                    self?.showsInAppNotification = true

                } else if case .loggedIn = previousDeviceState,
                          deviceState == .loggedOut || deviceState == .revoked
                {
                    self?.showsInAppNotification = false
                }
            })
        tunnelObserver.flatMap { tunnelManager.addObserver($0) }
    }
}
