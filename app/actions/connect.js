import { createAction } from 'redux-actions';

const connectionChange = createAction('CONNECTION_CHANGE');
const connect = (backend, addr) => () => backend.connect(addr);
const disconnect = (backend) => () => backend.disconnect();

export default { connect, disconnect, connectionChange };
