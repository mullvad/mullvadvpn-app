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
    'ca1.mullvad.net':  { 
      name: 'Canada',
      city: 'Ottawa',
      country: 'Canada',
      location: [45.421530, -75.697193]
    }, 
    'ca2.mullvad.net':  { 
      name: 'Canada (Quebec)',
      city: 'Quebec',
      country: 'Canada',
      location: [46.810811, -71.215439]
    }, 
    'da.mullvad.net':   { 
      name: 'Denmark',
      city: '‎Copenhagen',
      country: 'Denmark',
      location: [55.6760968, 12.5683371]
    }, 
    'de.mullvad.net':   { 
      name: 'Germany',
      city: 'Berlin',
      country: 'Germany',
      location: [52.52000659999999, 13.404954]
    }, 
    'lt.mullvad.net':   { 
      name: 'Lithuania',
      city: 'Vilnius',
      country: 'Lithuania',
      location: [54.6871555, 25.2796514]
    }, 
    'nl.mullvad.net':   { 
      name: 'The Netherlands',
      city: 'Amsterdam',
      country: 'The Netherlands',
      location: [52.3702157, 4.895167900000001]
    }, 
    'no.mullvad.net':   { 
      name: 'Norway',
      city: 'Oslo',
      country: 'Norway',
      location: [59.9138688, 10.7522454]
    }, 
    'ro.mullvad.net':   { 
      name: 'Romania',
      city: 'Bucharest',
      country: 'Canada',
      location: [44.4267674, 26.1025384]
    },
    'sg.mullvad.net':   { 
      name: 'Singapore',
      city: 'Singapore',
      country: 'Singapore',
      location: [1.352083, 103.819836]
    }, 
    'es.mullvad.net':   { 
      name: 'Spain',
      city: 'Madrid',
      country: 'Spain',
      location: [40.4167754, -3.7037902]
    }, 
    'se1.mullvad.net':  { 
      name: 'Sweden',
      city: 'Stockholm',
      country: 'Sweden',
      location: [59.32932349999999, 18.0685808] 
    }, 
    'ch.mullvad.net':   { 
      name: 'Switzerland',
      city: 'Zürich',
      country: 'Switzerland',
      location: [47.3768866, 8.541694]
    },
    'uk.mullvad.net':   { 
      name: 'United Kingdom',
      city: 'London',
      country: 'United Kingdom',
      location: [51.5073509, -0.1277583]
    }, 
    'us1.mullvad.net':  { 
      name: 'USA',
      city: 'New York',
      country: 'USA',
      location: [40.7127837, -74.0059413]
    }
  },
  defaultServer: 'se1.mullvad.net', // can be nearest, fastest or any of keys from .servers
  LoginState, ConnectionState
};
