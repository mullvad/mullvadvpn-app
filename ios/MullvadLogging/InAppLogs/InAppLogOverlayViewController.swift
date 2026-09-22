// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import UIKit

class InAppLogOverlayViewController: UIViewController {
    private let logView: InAppLogView
    private let interactor: InAppLogViewInteractor

    init(interactor: InAppLogViewInteractor) {
        self.interactor = interactor
        logView = InAppLogView(interactor: interactor)

        super.init(nibName: nil, bundle: nil)

        overrideUserInterfaceStyle = .light
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func loadView() {
        view = PassthroughView()
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.backgroundColor = .clear
        view.addSubview(logView)

        logView.onExportLogs = { [weak self] logString in
            let activityController = UIActivityViewController(
                activityItems: [logString],
                applicationActivities: nil
            )

            self?.present(activityController, animated: true)
        }
    }
}

private class PassthroughView: UIView {
    override func hitTest(_ point: CGPoint, with event: UIEvent?) -> UIView? {
        let hit = super.hitTest(point, with: event)
        return hit === self ? nil : hit
    }
}
