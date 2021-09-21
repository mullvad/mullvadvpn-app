//
//  TunnelObserver.swift
//  TunnelObserver
//
//  Created by pronebird on 19/08/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol TunnelObserver: AnyObject {
    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelState tunnelState: TunnelState)
    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelSettings tunnelInfo: TunnelInfo?)
    func tunnelManager(_ manager: TunnelManager, didFailWithError error: TunnelManager.Error)
}
