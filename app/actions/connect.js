import { createAction } from 'redux-actions';
import { ConnectionState } from '../constants';

const connectionChange = createAction('CONNECTION_CHANGE');
const connect = (backend, addr) => (dispatch, getState) => backend.connect(addr);
const disconnect = (backend) => (dispatch, getState) => backend.disconect();

export default { connect, disconnect, connectionChange };
