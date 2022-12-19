//
//  ChangeLogViewController.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-12-14.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

final class ChangeLogViewController: UIViewController, RootContainment {
    var preferredHeaderBarPresentation: HeaderBarPresentation {
        return .default
    }

    var prefersHeaderBarHidden: Bool {
        return false
    }

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    private lazy var contentView: ChangeLogContentView = {
        let contentView = ChangeLogContentView(frame: view.bounds)
        contentView.translatesAutoresizingMaskIntoConstraints = false
        return contentView
    }()

    private let changes: [String]

    init(for changes: [String]) {
        self.changes = changes

        super.init(nibName: nil, bundle: nil)
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.backgroundColor = .secondaryColor

        view.addSubview(contentView)

        NSLayoutConstraint.activate([
            contentView.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor),
            contentView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            contentView.bottomAnchor.constraint(equalTo: view.safeAreaLayoutGuide.bottomAnchor),
        ])

        contentView.continueButtonAction = { [unowned self] in
            ChangeLog.setVersion()
            self.dismiss(animated: true)
        }

        contentView.setChangeDescriptions(changes)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
