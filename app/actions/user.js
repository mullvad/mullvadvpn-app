import { createAction } from 'redux-actions';

export default {
  login: createAction('user.login', (account) => {
    return { account };
  })
};

