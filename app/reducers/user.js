import { handleActions } from 'redux-actions';

import actions from '../actions/user';

export default handleActions({
  [actions.loginSuccess]: (state, action) => {
    return { ...state, ...action.payload };
  },
  [actions.loginFailure]: (state, action) => {
    return { ...state, ...action.payload };
  }
}, {});
