//
//  GotaTun.swift
//  MullvadRustRuntime
//
//  Created by Emils on 31/10/2025.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadRustRuntimeProxy

public class GotaTun {
    private var handle: SwiftGotaTun = SwiftGotaTun(_0: nil)

    init(tunnelFileDescriptor: Int32, configuration: GotaTunConfig) {
        mullvad_ios_gotatun_start(&handle, configruation.handle, tunnelFileDescriptor, )

    }

    deinit {

    }
}

public class GotaTunConfig {
    fileprivate var handle = mullvad_ios_gotatun_config_new()

    func addEntry() {

    }

    func addExit(privateKey: Data, preSharedKey: Data? = nil, publicKey: Data, endpoint: String) {
        mullvad_ios_gotatun_config_set_exit(handle,  privateKey, preSharedKey, publicKey, endpoint)
    }

    
    deinit {
        mullvad_ios_gotatun_config_drop(handle)
    }

}
