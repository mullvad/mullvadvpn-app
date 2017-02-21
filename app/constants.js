import Enum from './lib/enum';

const LoginState = Enum('none', 'connecting', 'failed', 'ok');
const ConnectionState = Enum('disconnected', 'connecting', 'connected', 'failed');

module.exports = {
  links: {
    createAccount: 'https://mullvad.net/account/create/',
    faq: 'https://mullvad.net/faq/',
    guides: 'https://mullvad.net/guides/',
    supportEmail: 'mailto:support@mullvad.net'
  },
  servers: {
    'ca1.mullvad.net':  { name: 'Canada' }, 
    'ca2.mullvad.net':  { name: 'Canada (Quebec)' }, 
    'da.mullvad.net':   { name: 'Denmark' }, 
    'de.mullvad.net':   { name: 'Germany' }, 
    'lt.mullvad.net':   { name: 'Lithuania' }, 
    'nl.mullvad.net':   { name: 'The Netherlands' }, 
    'no.mullvad.net':   { name: 'Norway' }, 
    'ro.mullvad.net':   { name: 'Romania' },
    'sg.mullvad.net':   { name: 'Singapore' }, 
    'es.mullvad.net':   { name: 'Spain' }, 
    'se1.mullvad.net':  { name: 'Sweden', isDefault: true }, 
    'ch.mullvad.net':   { name: 'Switzerland' },
    'uk.mullvad.net':   { name: 'United Kingdom' }, 
    'us1.mullvad.net':  { name: 'USA' }
  },
  LoginState, ConnectionState
};
