//
//  IntentHandler.swift
//  ToggleTunnel
//
//  Created by Nikolay Davydov on 07.04.2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Intents
import NetworkExtension

final class IntentHandler: INExtension, ToggleTunnelIntentHandling {
    
    func handle(intent: ToggleTunnelIntent, completion: @escaping (ToggleTunnelIntentResponse) -> Void) {
        
        guard let isOn = intent.connection?.boolValue else {
            completion(.init(code: .failure, userActivity: nil))
            return
        }
        
        setTunnelConnection(isOn: isOn, completion: completion)
    }
    
    private func setTunnelConnection(isOn: Bool, completion: @escaping (ToggleTunnelIntentResponse) -> Void) {
        
        NETunnelProviderManager.loadAllFromPreferences { managers, error in
            
            guard let manager = managers?.first else {
                completion(.init(code: .failure, userActivity: nil))
                return
            }
            
            guard isOn else {
                manager.connection.stopVPNTunnel()
                completion(.init(code: .success, userActivity: nil))
                return
            }
            
            do {
                try manager.connection.startVPNTunnel()
                completion(.init(code: .success, userActivity: nil))
            } catch {
                completion(.init(code: .failure, userActivity: nil))
            }
        }
    }
}
