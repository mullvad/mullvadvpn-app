//
//  BoringTunWrapper.swift
//  BoringtunWrapper
//
//  Created by Marco Nikic on 2025-04-25.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import BoringTunProxy

public class BoringTunWrapper {
    public init() {}

    public func createTunnel() {
        print("Hello")
        new_tunnel("helo", "hello", "hello", 0, 0)
    }
}
