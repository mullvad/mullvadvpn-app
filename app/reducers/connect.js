import { handleActions } from 'redux-actions';
import { ConnectionState } from '../enums';
import actions from '../actions/connect';

const initialState = {
  status: ConnectionState.disconnected,
  isOnline: true,
  serverAddress: null,
  clientIp: null
};

export default handleActions({ 
  [actions.connectionChange]: (state, action) => { 
    return { ...state, ...action.payload };
  } 
}, initialState);
