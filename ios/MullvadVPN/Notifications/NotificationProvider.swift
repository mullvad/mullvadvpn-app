//
//  NotificationProvider.swift
//  MullvadVPN
//
//  Created by pronebird on 09/12/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UserNotifications

/// Notification provider delegate primarily used by `NotificationManager`.
protocol NotificationProviderDelegate: AnyObject {
    func notificationProviderDidInvalidate(_ notificationProvider: NotificationProvider)
    func notificationProvider(_ notificationProvider: NotificationProvider, didReceiveAction actionIdentifier: String)
}

/// Base class for all notification providers.
class NotificationProvider: NotificationProviderProtocol {
    weak var delegate: NotificationProviderDelegate?

    /**
     Provider identifier.

     Override in subclasses and make sure each provider has unique identifier. It's preferred that identifiers use
     reverse domain name, for instance: `com.example.app.ProviderName`.
     */
    var identifier: String {
        return "default"
    }

    /**
     Send action to notification manager delegate.

     Usually in response to user interacting with notification banner, i.e by tapping a button. Use different action
     identifiers if notification offers more than one action that user can perform.
     */
    func sendAction(_ actionIdentifier: String = UNNotificationDefaultActionIdentifier) {
        dispatchOnMain {
            self.delegate?.notificationProvider(self, didReceiveAction: actionIdentifier)
        }
    }

    /**
     This method tells notification manager to re-evalute the notification content.
     Call this method when notification provider wants to change the content it presents.
     */
    func invalidate() {
        dispatchOnMain {
            self.delegate?.notificationProviderDidInvalidate(self)
        }
    }

    private func dispatchOnMain(_ block: @escaping () -> Void) {
        if Thread.isMainThread {
            block()
        } else {
            DispatchQueue.main.async(execute: block)
        }
    }
}
