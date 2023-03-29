//
//  SafariCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 29/03/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import SafariServices

class SafariCoordinator: Coordinator, Presentable, SFSafariViewControllerDelegate {
    var didFinish: (() -> Void)?

    var presentedViewController: UIViewController {
        return safariController
    }

    private let safariController: SFSafariViewController

    init(url: URL) {
        safariController = SFSafariViewController(url: url)
        super.init()
        
        safariController.delegate = self
    }

    func safariViewControllerDidFinish(_ controller: SFSafariViewController) {
        dismiss(animated: true) {
            self.didFinish?()
        }
    }

    func safariViewControllerWillOpenInBrowser(_ controller: SFSafariViewController) {
        dismiss(animated: false) {
            self.didFinish?()
        }
    }
}
