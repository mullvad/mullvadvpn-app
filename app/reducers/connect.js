import { handleActions } from 'redux-actions';
import { ConnectionState } from '../constants';
import actions from '../actions/connect';

const initialState = {
  status: ConnectionState.disconnected,
  serverAddress: null,
  clientIp: null
};

export default handleActions({ 
  [actions.connectionChange]: (state, action) => { 
    return { ...state, ...action.payload };
  } 
}, initialState);
