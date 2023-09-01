//
//  AlertPresenter.swift
//  MullvadVPN
//
//  Created by pronebird on 04/06/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Routing

struct AlertPresenter {
    let context: any Presenting

    func showAlert(presentation: AlertPresentation, animated: Bool) {
        context.applicationRouter?.presentAlert(
            route: .alert,
            metadata: AlertMetadata(presentation: presentation, context: context)
        )
    }

    func dismissAlert(animated: Bool) {
        context.applicationRouter?.dismiss(.alert, animated: animated)
    }
}

extension ApplicationRouter {
    func presentAlert(route: RouteType, metadata: Any) {
        present(route, animated: true, metadata: metadata)
    }
}
