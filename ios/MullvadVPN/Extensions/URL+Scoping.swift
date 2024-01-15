//
//  URL+Scoping.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-02.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension URL {
    func securelyScoped(_ completionHandler: (Self?) -> Void) {
        if startAccessingSecurityScopedResource() {
            completionHandler(self)
            stopAccessingSecurityScopedResource()
        } else {
            completionHandler(nil)
        }
    }
}
