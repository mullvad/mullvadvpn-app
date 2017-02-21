import { handleActions } from 'redux-actions';

import { servers } from '../constants';

import actions from '../actions/settings';

const addrs = Object.keys(servers);
const defaultServer = addrs.find((k) => servers[k].isDefault) || servers[addrs[0]] || {};

const initialState = {
  autoSecure: false,
  preferredServer: defaultServer
};

export default handleActions({
  [actions.updateSettings]: (state, action) => {
    return { ...state, ...action.payload };
  }
}, initialState);
