import { replace } from 'react-router-redux';
import Backend from './backend';

/**
 * Add listeners to translate backend events to react router actions
 *
 * @export
 * @param {Backend} backend
 * @param {Redux.Store} store
 */
export default function mapBackendEventsToRouter(backend, store) {
  // redirect user to main screen after login
  backend.on(Backend.EventType.login, (account, error) => {
    if(error) { return; } // no-op on error

    setTimeout(() => {
      const { settings } = store.getState();

      // auto-connect only if autoSecure is on
      if(settings.autoSecure) {
        const server = backend.serverInfo(settings.preferredServer);
        backend.connect(server.address);
      }

      store.dispatch(replace('/connect'));
    }, 1000);
  });

  // redirect user to login page on logout
  backend.on(Backend.EventType.logout, () => store.dispatch(replace('/')));
}