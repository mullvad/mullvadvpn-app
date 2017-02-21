import { handleActions } from 'redux-actions';
import { defaultServer } from '../constants';
import actions from '../actions/settings';

const initialState = {
  autoSecure: false,
  preferredServer: defaultServer
};

export default handleActions({
  [actions.updateSettings]: (state, action) => {
    return { ...state, ...action.payload };
  }
}, initialState);
