//
//  AlertPresenter.swift
//  MullvadVPN
//
//  Created by pronebird on 04/06/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Routing

struct AlertPresenter {
    let coordinator: Coordinator

    func showAlert(presentation: AlertPresentation, animated: Bool) {
        coordinator.applicationRouter?.present(.alert(presentation), animated: animated)
    }
}
