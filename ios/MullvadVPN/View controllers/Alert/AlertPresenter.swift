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
        context.applicationRouter?.present(.alert(presentation, context), animated: animated)
    }

    func dismissAlert(presentation: AlertPresentation, animated: Bool) {
        context.applicationRouter?.dismiss(.alert(presentation, context), animated: animated)
    }
}
