//
//  LogOverlayViewController.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-04-24.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import UIKit

class LogOverlayViewController: UIViewController {
    private let logView: LogView
    private let viewModel: LogViewModel

    init(viewModel: LogViewModel) {
        self.viewModel = viewModel
        logView = LogView(viewModel: viewModel)

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

        logView.onShareLogs = { [weak self] logString in
            let activityController = UIActivityViewController(
                activityItems: [logString],
                applicationActivities: nil
            )

//            activityController.popoverPresentationController?.barButtonItem = navigationItem.leftBarButtonItem

            self?.present(activityController, animated: true)
        }
    }
}

class PassthroughWindow: UIWindow {
    override func hitTest(_ point: CGPoint, with event: UIEvent?) -> UIView? {
        let hit = super.hitTest(point, with: event)
        return hit === self ? nil : hit
    }
}

private class PassthroughView: UIView {
    override func hitTest(_ point: CGPoint, with event: UIEvent?) -> UIView? {
        let hit = super.hitTest(point, with: event)
        return hit === self ? nil : hit
    }
}
