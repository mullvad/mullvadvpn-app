import { handleActions } from 'redux-actions';

import actions from '../actions/user';

export default handleActions({
  [actions.loginChange]: (state, action) => {
    console.log(action.payload);
    return { ...state, ...action.payload };
  }
}, {});
