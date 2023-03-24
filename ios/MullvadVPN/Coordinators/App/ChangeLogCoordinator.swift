//
//  ChangeLogCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 24/03/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import UIKit

final class ChangeLogCoordinator: Coordinator {
    private let logger = Logger(label: "ChangeLogCoordinator")

    let navigationController: RootContainerViewController

    var didFinish: ((ChangeLogCoordinator) -> Void)?

    init(navigationController: RootContainerViewController) {
        self.navigationController = navigationController
    }

    func start(animated: Bool) {
        let controller = ChangeLogViewController()

        controller.setApplicationVersion(Bundle.main.productVersion)

        do {
            let string = try ChangeLog.readFromFile()

            controller.setChangeLogText(string)
        } catch {
            logger.error(error: error, message: "Cannot read changelog from bundle.")
        }

        controller.onFinish = { [weak self] in
            guard let self = self else { return }

            ChangeLog.markAsSeen()

            self.didFinish?(self)
        }

        navigationController.pushViewController(controller, animated: animated)
    }
}
