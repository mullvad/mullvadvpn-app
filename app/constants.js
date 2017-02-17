import Enum from './lib/enum';

const LoginState = Enum('none', 'connecting', 'failed', 'ok');

module.exports = {
  links: {
    createAccount: 'https://mullvad.net/account/create/',
    faq: 'https://mullvad.net/faq/',
    guides: 'https://mullvad.net/guides/',
    supportEmail: 'mailto:support@mullvad.net'
  },
  LoginState
};
