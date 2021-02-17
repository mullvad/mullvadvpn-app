/* SPDX-License-Identifier: Apache-2.0
 *
 * Copyright (C) 2017-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.
 * Copyright (C) 2021 Mullvad VPN AB. All Rights Reserved.
 */

package logging

// #include <stdlib.h>
// #include <sys/types.h>
// #ifndef WIN32
// #define __stdcall
// #endif
// typedef void (__stdcall *LogSink)(unsigned int, const char *, void *);
// static void callLogSink(void *logSink, int level, const char *message, void *context)
// {
//   ((LogSink)logSink)((unsigned int)level, message, context);
// }
import "C"

import (
	"log"
	"unsafe"

	"golang.zx2c4.com/wireguard/device"
)

// Define type aliases.
type LogSink = unsafe.Pointer
type LogContext = unsafe.Pointer

type Logger struct {
	sink    LogSink
	context LogContext
	level   C.int
}

func (l *Logger) Write(message []byte) (int, error) {
	msg := C.CString(string(message))
	C.callLogSink(l.sink, l.level, msg, l.context)
	C.free(unsafe.Pointer(msg))
	return len(message), nil
}

func NewLogger(logSink LogSink, logContext LogContext) *device.Logger {
	logger := new(device.Logger)

	logger.Debug = log.New(
		&Logger{sink: logSink, context: logContext, level: device.LogLevelDebug},
		"",
		0,
	)
	logger.Info = log.New(
		&Logger{sink: logSink, context: logContext, level: device.LogLevelInfo},
		"",
		0,
	)
	logger.Error = log.New(
		&Logger{sink: logSink, context: logContext, level: device.LogLevelError},
		"",
		0,
	)

	return logger
}
