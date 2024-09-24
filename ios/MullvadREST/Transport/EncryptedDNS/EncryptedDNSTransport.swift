//
//  EncryptedDNSTransport.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-09-19.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//
import Foundation
import MullvadRustRuntime
import MullvadTypes

public final class EncryptedDNSTransport: RESTTransport {
    public var name: String {
        "encrypted-dns-url-session"
    }

    /// The `URLSession` used to send requests via `encryptedDNSProxy`
    public let urlSession: URLSession
    private let encryptedDnsProxy: EncryptedDNSProxy
    private let dispatchQueue = DispatchQueue(label: "net.mullvad.EncryptedDNSTransport")
    private var dnsProxyTask: URLSessionTask!

    public init(urlSession: URLSession) {
        self.urlSession = urlSession
        self.encryptedDnsProxy = EncryptedDNSProxy()
    }

    public func stop() {
        dispatchQueue.async { [weak self] in
            self?.encryptedDnsProxy.stop()
            self?.dnsProxyTask = nil
        }
    }

    /// Sends a request via the encrypted DNS proxy.
    ///
    /// Starting the proxy can take a very long time due to domain resolution
    /// Cancellation will only take place if the data task was already started, at which point,
    /// most of the time starting the DNS proxy was already spent.
    public func sendRequest(
        _ request: URLRequest,
        completion: @escaping (Data?, URLResponse?, (any Error)?) -> Void
    ) -> any Cancellable {
        dispatchQueue.async { [weak self] in
            guard let self else { return }
            do {
                try self.encryptedDnsProxy.start()
            } catch {
                completion(nil, nil, error)
                return
            }

            var urlRequestCopy = request
            urlRequestCopy.url = request.url.flatMap { url in
                var components = URLComponents(url: url, resolvingAgainstBaseURL: false)
                components?.host = "127.0.0.1"
                components?.port = Int(self.encryptedDnsProxy.localPort())
                return components?.url
            }

            let wrappedCompletionHandler: (Data?, URLResponse?, (any Error)?)
                -> Void = { [weak self] data, response, maybeError in
                    if maybeError != nil {
                        self?.encryptedDnsProxy.stop()
                    }
                    // Do not call the completion handler if the request was cancelled in flight
                    if let cancelledError = maybeError as? URLError, cancelledError.code == .cancelled {
                        return
                    }

                    completion(data, response, maybeError)
                }

            let dataTask = urlSession.dataTask(with: urlRequestCopy, completionHandler: wrappedCompletionHandler)
            dataTask.resume()
            dnsProxyTask = dataTask
        }

        return AnyCancellable { [weak self] in
            self?.dispatchQueue.async {
                self?.dnsProxyTask.cancel()
            }
        }
    }
}
