import { handleActions } from 'redux-actions';
import actions from '../actions/user';
import { LoginState } from '../enums';

const initialState = {
  account: null,
  paidUntil: null, // ISO8601
  location: [0, 0],
  country: null,
  city: null,
  status: LoginState.none,
  error: null
};

export default handleActions({
  [actions.loginChange]: (state, action) => {
    return { ...state, ...action.payload };
  }
}, initialState);
