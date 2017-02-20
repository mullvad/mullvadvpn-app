import { handleActions } from 'redux-actions';

import actions from '../actions/settings';

const initialState = {
  autoSecure: false,
  preferredServer: 'Sweden'
};

export default handleActions({
  [actions.updateSettings]: (state, action) => {
    return { ...state, ...action.payload };
  }
}, initialState);
