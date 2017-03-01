import { replace } from 'react-router-redux';
import Backend from './backend';

export default function mapBackendEventsToRouter(backend, store) {
  // redirect user to main screen after login
  backend.on(Backend.EventType.login, (account, error) => {
    if(error) { return; } // no-op on error

    setTimeout(() => {
      const { settings } = store.getState();
      const server = backend.serverInfo(settings.preferredServer);
      backend.connect(server.address);
      store.dispatch(replace('/connect'));
    }, 1000);
  });

  // redirect user to login page on logout
  backend.on(Backend.EventType.logout, () => store.dispatch(replace('/')));
}