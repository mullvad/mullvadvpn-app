//
//  ProxyConfigurationViewControllerDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 23/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol ProxyConfigurationViewControllerDelegate: AnyObject {
    func controllerShouldShowProtocolPicker(_ controller: ProxyConfigurationViewController)
    func controllerShouldShowShadowsocksCipherPicker(_ controller: ProxyConfigurationViewController)
}
