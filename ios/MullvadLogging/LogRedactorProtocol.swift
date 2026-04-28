//
//  Untitled.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-28.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

public protocol LogRedactorProtocol: Sendable {
    func redact(_ input: String) -> String
}
