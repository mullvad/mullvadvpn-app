//
//  NotificationProvider.swift
//  MullvadVPN
//
//  Created by pronebird on 09/12/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Notification provider delegate primarily used by `NotificationManager`.
protocol NotificationProviderDelegate: AnyObject {
    func notificationProviderDidInvalidate(_ notificationProvider: NotificationProvider)
}

/// Base class for all notification providers.
class NotificationProvider: NotificationProviderProtocol {
    weak var delegate: NotificationProviderDelegate?

    var identifier: String {
        return "default"
    }

    func invalidate() {
        let executor = {
            self.delegate?.notificationProviderDidInvalidate(self)
            return
        }

        if Thread.isMainThread {
            executor()
        } else {
            DispatchQueue.main.async(execute: executor)
        }
    }
}
