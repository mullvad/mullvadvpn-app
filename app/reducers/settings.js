import { handleActions } from 'redux-actions';
import { defaultServer } from '../config';
import actions from '../actions/settings';

const initialState = {
  autoSecure: true,
  preferredServer: defaultServer
};

export default handleActions({
  [actions.updateSettings]: (state, action) => {
    return { ...state, ...action.payload };
  }
}, initialState);
