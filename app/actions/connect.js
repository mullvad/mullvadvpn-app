import { clipboard } from 'electron';
import { createAction } from 'redux-actions';

/** Action for changing connection state */
const connectionChange = createAction('CONNECTION_CHANGE');

/** Action for connecting to server */
const connect = (backend, addr) => () => backend.connect(addr);

/** Action for disconnecting from server */
const disconnect = (backend) => () => backend.disconnect();

/** Action for copying IP address in memory */
const copyIPAddress = () => {
  return (_, getState) => {
    const ip = getState().connect.clientIp;
    if(typeof(ip) === 'string') {
      clipboard.writeText(ip);
    }
  };
};

export default { connect, disconnect, copyIPAddress, connectionChange };
