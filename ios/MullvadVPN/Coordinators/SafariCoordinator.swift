// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import Routing
import SafariServices

@MainActor
class SafariCoordinator: Coordinator, Presentable, @preconcurrency SFSafariViewControllerDelegate {
    nonisolated(unsafe) var didFinish: (@Sendable () -> Void)?

    var presentedViewController: UIViewController {
        safariController
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
