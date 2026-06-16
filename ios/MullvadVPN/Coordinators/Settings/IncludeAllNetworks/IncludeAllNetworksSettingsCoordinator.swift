//
//  IncludeAllNetworksSettingsCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-01-16.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Logging
import Routing
import SwiftUI

class IncludeAllNetworksSettingsCoordinator: Coordinator, SettingsChildCoordinator, Presentable, Presenting {
    private lazy var logger = Logger(label: "IncludeAllNetworksSettingsCoordinator")
    private let navigationController: UINavigationController
    private let viewModel: IncludeAllNetworksSettingsViewModelImpl
    private let route: AppRoute

    var presentedViewController: UIViewController {
        navigationController
    }

    var didFinish: ((IncludeAllNetworksSettingsCoordinator) -> Void)?

    init(
        navigationController: UINavigationController,
        route: AppRoute,
        viewModel: IncludeAllNetworksSettingsViewModelImpl
    ) {
        self.navigationController = navigationController
        self.route = route
        self.viewModel = viewModel

        super.init()
    }

    func start(animated: Bool) {
        var view = IncludeAllNetworksSettingsView(viewModel: self.viewModel)
        view.showIssuesInfo = showIssuesInfo

        let host = UIHostingController(rootView: view)
        host.title = NSLocalizedString("Force all apps", comment: "")
        host.view.setAccessibilityIdentifier(.includeAllNetworksView)
        customiseNavigation(on: host)

        navigationController.pushViewController(host, animated: animated)
    }

    private func customiseNavigation(on viewController: UIViewController) {
        if route == .includeAllNetworks {
            navigationController.navigationItem.largeTitleDisplayMode = .always
            navigationController.navigationBar.prefersLargeTitles = true

            let doneButton = UIBarButtonItem(
                systemItem: .done,
                primaryAction: UIAction(handler: { [weak self] _ in
                    guard let self else { return }
                    didFinish?(self)
                })
            )
            viewController.navigationItem.rightBarButtonItem = doneButton
        }
    }

    private func showIssuesInfo() {
        let featuresString = NSLocalizedString(
            "**AirDrop, AirPlay, CarPlay, Continuity Camera, Handoff, Handover, NameDrop, iMessage, Screen Mirroring** and **Personal Hotspot**.",
            comment: "Feature names in **double asterisks** will be in semibold font."
        )

        var attributedFeaturesString =
            (try? AttributedString(markdown: featuresString))
            ?? AttributedString(featuresString.replacingOccurrences(of: "*", with: ""))
        for run in attributedFeaturesString.runs
        where run.inlinePresentationIntent?.contains(.stronglyEmphasized) == true {
            attributedFeaturesString[run.range].font = .mullvadTinySemiBold
        }

        let aboutView = AboutView(
            header: NSLocalizedString("Known issues", comment: ""),
            preamble: nil,
            paragraphs: [
                AttributedString(NSLocalizedString("iOS features known to be affected:", comment: ""))
                    + AttributedString("\n")
                    + attributedFeaturesString,
                AttributedString(
                    String(
                        format: NSLocalizedString(
                            "Enabling %@ has shown to reduce the number of issues encountered when using %@, regardless if you are "
                                + "using WiFi or a cellular network.",
                            comment: "Variables are 'Local network sharing' and 'Force all apps', respectively"
                        ),
                        NSLocalizedString("Local network sharing", comment: ""),
                        NSLocalizedString("Force all apps", comment: "")
                    )
                ),
            ]
        )

        let host = UIHostingController(rootView: aboutView)
        let customNavigationController = CustomNavigationController(rootViewController: host)

        host.navigationItem.rightBarButtonItem = UIBarButtonItem(
            systemItem: .done,
            primaryAction: UIAction(handler: { _ in
                customNavigationController.dismiss(animated: true)
            })
        )

        navigationController.present(customNavigationController, animated: true)
    }
}
