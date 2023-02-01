//
//  LogFileOutputStream.swift
//  MullvadVPN
//
//  Created by pronebird on 02/08/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Interval used for reopening the log file descriptor in the event of failure to open it in
/// the first place, or when writing to it.
private let reopenFileLogInterval: TimeInterval = 5

class LogFileOutputStream: TextOutputStream {
    private let queue = DispatchQueue(label: "LogFileOutputStreamQueue", qos: .utility)

    private let fileURL: URL
    private let encoding: String.Encoding
    private let maxBufferCapacity: Int

    private var state: State = .closed {
        didSet {
            switch (oldValue, state) {
            case (.opened, .waitingToReopen), (.closed, .waitingToReopen):
                startTimer()

            case (.waitingToReopen, .opened), (.waitingToReopen, .closed):
                stopTimer()

            default:
                break
            }
        }
    }

    private var timer: DispatchSourceTimer?
    private var buffer = Data()

    private enum State {
        case closed
        case opened(FileHandle)
        case waitingToReopen
    }

    init(fileURL: URL, encoding: String.Encoding = .utf8, maxBufferCapacity: Int = 16 * 1024) {
        self.fileURL = fileURL
        self.encoding = encoding
        self.maxBufferCapacity = maxBufferCapacity
    }

    deinit {
        stopTimer()
    }

    func write(_ string: String) {
        queue.async {
            self.writeNoQueue(string)
        }
    }

    private func writeNoQueue(_ string: String) {
        guard let data = string.data(using: encoding) else { return }

        switch state {
        case .closed:
            do {
                let fileHandle = try openFile()
                state = .opened(fileHandle)
                try write(fileHandle: fileHandle, data: data)
            } catch {
                bufferData(data)
                state = .waitingToReopen
            }

        case let .opened(fileHandle):
            do {
                try write(fileHandle: fileHandle, data: data)
            } catch {
                bufferData(data)
                state = .waitingToReopen
            }

        case .waitingToReopen:
            bufferData(data)
        }
    }

    @discardableResult private func write(fileHandle: FileHandle, data: Data) throws -> Int {
        let bytesWritten = data.withUnsafeBytes { buffer -> Int in
            guard let ptr = buffer.baseAddress else { return 0 }

            return Darwin.write(fileHandle.fileDescriptor, ptr, buffer.count)
        }

        if bytesWritten == -1 {
            let code = POSIXErrorCode(rawValue: errno)!
            throw POSIXError(code)
        } else {
            return bytesWritten
        }
    }

    private func openFile() throws -> FileHandle {
        let oflag: Int32 = O_WRONLY | O_CREAT
        let mode: mode_t = S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH

        let fd = fileURL.path.withCString { Darwin.open($0, oflag, mode) }

        if fd == -1 {
            let code = POSIXErrorCode(rawValue: errno)!
            throw POSIXError(code)
        } else {
            return FileHandle(fileDescriptor: fd, closeOnDealloc: true)
        }
    }

    private func startTimer() {
        timer?.cancel()

        let timer = DispatchSource.makeTimerSource(queue: queue)
        timer.setEventHandler { [weak self] in
            self?.reopenFile()
        }
        timer.schedule(
            wallDeadline: .now() + reopenFileLogInterval,
            repeating: reopenFileLogInterval
        )
        timer.activate()

        self.timer = timer
    }

    private func stopTimer() {
        timer?.cancel()
        timer = nil
    }

    private func reopenFile() {
        do {
            let fileHandle = try openFile()

            // Write a message indicating that the file was reopened.
            let messageData =
                "<Log file re-opened after failure. Buffered \(buffer.count) bytes of messages>\n"
                    .data(using: encoding, allowLossyConversion: true)!
            try write(fileHandle: fileHandle, data: messageData)

            // Write all buffered messages.
            if !buffer.isEmpty {
                try write(fileHandle: fileHandle, data: buffer)
                buffer.removeAll()
            }

            state = .opened(fileHandle)
        } catch {
            state = .waitingToReopen
        }
    }

    private func bufferData(_ data: Data) {
        buffer.append(data)

        if buffer.count > maxBufferCapacity {
            buffer.removeFirst(buffer.count - maxBufferCapacity)
        }
    }
}
