import { createAction } from 'redux-actions';

const loginSuccess = createAction('user.loginSuccess', (account) => {
  return { account, loggedIn: true };
});

const loginFailure = createAction('user.loginFailure', (account, error) => {
  return { account, error, loggedIn: false };
});

const login = (backend, account) => {
  return async (dispatch) => {
    try {
      await backend.login(account);
      dispatch(loginSuccess(account));
    } catch(e) {
      dispatch(loginFailure(account, e));
    }
  };
};

export default { login, loginSuccess, loginFailure };

