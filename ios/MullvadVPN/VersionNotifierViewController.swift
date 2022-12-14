//
//  VersionNotifierViewController.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-12-14.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

final class VersionNotifierViewController: UIViewController, RootContainment {
    var preferredHeaderBarPresentation: HeaderBarPresentation {
        return .default
    }

    var prefersHeaderBarHidden: Bool {
        return false
    }

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    private lazy var contentView: VersionNotifierContentView = {
        let contentView = VersionNotifierContentView()
        contentView.translatesAutoresizingMaskIntoConstraints = false
        return contentView
    }()

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
            self.dismiss(animated: true)
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
            self.contentView.setChangeDescriptions(["Sajad", "HiSAJDKASHLDJKHASJLKDHAJSKDHLASKJHDJALKSBNDALKSJNDNASDNKASLNJKDNALJSKNDLASNDJLASNDLJANSLDKNASLKJDNASKLNDASKDNASKLDNASJKDNASLK"])
        }
    }
}
