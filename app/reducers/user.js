import { handleActions } from 'redux-actions';

export default handleActions({
  USER_LOGIN: (state, action) => {
    return { ...state, ...action.payload };
  }
}, {});
