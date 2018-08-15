// @flow
import log from 'electron-log';
import { ipcRenderer } from 'electron';

/**
 * Executes the provided closure when application is about to quit.
 * It works in tandem with ShutdownCoordinator and can only be used in renderer process.
 * The shutdown will be postponed until the Promise returned from closure is resolved.
 * ShutdownCoordinator will force the shutdown if the returned Promise is not resolved within
 * the allocated time (see SHUTDOWN_TIMEOUT).
 */
export default function setShutdownHandler(handler: () => Promise<void>) {
  ipcRenderer.once('app-shutdown', async (event) => {
    try {
      await handler();
    } catch (error) {
      log.warn(`An error occurred in shutdown handler: ${error.message}`);
    }
    event.sender.send('app-shutdown-reply');
  });
}
