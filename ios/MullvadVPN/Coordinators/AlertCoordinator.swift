// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadLogging
import Routing
import UIKit

final class AlertCoordinator: Coordinator, Presentable {
    private var alertController: AlertViewController?
    private let presentation: AlertPresentation

    var didFinish: (() -> Void)?

    var presentedViewController: UIViewController {
        return alertController!
    }

    init(presentation: AlertPresentation) {
        self.presentation = presentation
    }

    func start() {
        alertController = AlertViewController(presentation: presentation)

        alertController?.onDismiss = { [weak self] in
            self?.didFinish?()
        }
    }
}
